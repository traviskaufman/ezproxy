#!/usr/bin/env bash
set -euo pipefail

usage="usage: $0 <rounds> <label>=<ezproxy binary>..."
rounds="${1:?$usage}"
shift
[ "$#" -gt 0 ] || { echo "$usage" >&2; exit 1; }

benchmarks_dir="$(cd "$(dirname "$0")" && pwd)"
config="$benchmarks_dir/../ezproxy-core/example-configs/simple.txt"
port=5599
base_url="http://127.0.0.1:${port}"
keepalive_requests=100000
concurrent_requests=200000
new_connection_requests=1500
server_pid=""

stop_server() {
  [ -n "$server_pid" ] || return 0
  kill "$server_pid" 2>/dev/null || true
  wait "$server_pid" 2>/dev/null || true
  server_pid=""
}
trap stop_server EXIT

start_server() {
  if nc -z 127.0.0.1 "$port" 2>/dev/null; then
    echo "port $port is already in use" >&2
    exit 1
  fi
  "$1" "$config" --port "$port" &
  server_pid=$!
  until nc -z 127.0.0.1 "$port" 2>/dev/null; do
    kill -0 "$server_pid" 2>/dev/null || { echo "$1 exited before listening" >&2; exit 1; }
    sleep 0.05
  done
}

warm_up() {
  ab -q -k -c 4 -n 20000 "${base_url}/?q=npm%20file%20finder" >/dev/null 2>&1
}

run_scenario() {
  local scenario="$1" requests="$2" url="${base_url}$3"
  shift 3
  local status before after report
  status="$(curl -s -o /dev/null -w '%{http_code}' "$url")"
  [ "$status" = 302 ] || { echo "$label: $url returned $status, expected 302" >&2; exit 1; }
  before="$(python3 "$benchmarks_dir/process_counters.py" "$server_pid")"
  report="$(ab -q -n "$requests" "$@" "$url" 2>&1)"
  after="$(python3 "$benchmarks_dir/process_counters.py" "$server_pid")"
  awk -v label="$label" -v scenario="$scenario" -v round="$round" -v requests="$requests" \
    -v before="$before" -v after="$after" '
    /^Requests per second/ { redirects_per_second = $4 }
    /^Time per request/ && /\(mean\)/ { mean_latency_us = $4 * 1000 }
    /^Failed requests/ { failed_requests = $3 }
    END {
      split(before, counters_before, " ")
      split(after, counters_after, " ")
      printf "%s,%s,%s,%s,%.1f,%.2f,%.0f,%s\n", label, scenario, round,
        redirects_per_second, mean_latency_us,
        (counters_after[1] - counters_before[1]) / requests / 1000,
        (counters_after[2] - counters_before[2]) / requests, failed_requests
    }' <<<"$report"
}

echo "label,scenario,round,redirects_per_second,mean_latency_us,server_cpu_us_per_redirect,server_instructions_per_redirect,failed_requests"
for round in $(seq 1 "$rounds"); do
  for build in "$@"; do
    label="${build%%=*}"
    start_server "${build#*=}"
    warm_up
    run_scenario static-keepalive-c1 "$keepalive_requests" "/?q=m" -k -c 1
    run_scenario args-keepalive-c1 "$keepalive_requests" "/?q=npm%20file%20finder" -k -c 1
    run_scenario fallback-keepalive-c1 "$keepalive_requests" "/?q=best%20restaurants%20nyc" -k -c 1
    run_scenario args-keepalive-c32 "$concurrent_requests" "/?q=npm%20file%20finder" -k -c 32
    run_scenario static-newconn-c1 "$new_connection_requests" "/?q=m" -c 1
    stop_server
  done
done
