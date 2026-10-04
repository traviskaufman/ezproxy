---
name: benchmarking-redirects
description: Measures how fast the ezproxy server answers redirects and compares two or more builds on redirects per second, latency, server CPU time, and server instructions per redirect. Use whenever a change to ezproxy-core might affect speed, when asked to benchmark, measure, profile, optimize, or speed up ezproxy, when asked whether a change made redirects faster or slower, and before claiming any performance improvement or regression, even if the user never says "benchmark".
---

# Benchmarking redirects

`benchmarks/redirects.sh` starts each build on port 5599 with `ezproxy-core/example-configs/simple.txt`, drives it with `ab`, and prints one CSV row per build, scenario, and round. `benchmarks/summarize.py` turns the CSV into medians with the change against the first build.

Requires macOS: `ab`, `nc`, `curl`, `python3`, and the `proc_pid_rusage` call that `benchmarks/process_counters.py` reads CPU time and instruction counts from.

## Workflow

Run every command from the repository root.

1. Build the baseline from the commit being compared against, and copy it out of `target/`:

   ```sh
   git worktree add /tmp/ezproxy-baseline-src <baseline-commit>
   cargo build --release --target aarch64-apple-darwin \
     --manifest-path /tmp/ezproxy-baseline-src/ezproxy-core/Cargo.toml \
     --target-dir ezproxy-core/target
   cp ezproxy-core/target/aarch64-apple-darwin/release/ezproxy /tmp/ezproxy-baseline
   git worktree remove /tmp/ezproxy-baseline-src
   ```

2. Build the candidate from the working tree:

   ```sh
   (cd ezproxy-core && cargo build --release --target aarch64-apple-darwin)
   ```

3. Run the benchmark, baseline first, then summarize:

   ```sh
   benchmarks/redirects.sh 5 \
     baseline=/tmp/ezproxy-baseline \
     candidate=ezproxy-core/target/aarch64-apple-darwin/release/ezproxy \
     > /tmp/redirects.csv
   python3 benchmarks/summarize.py /tmp/redirects.csv
   ```

   Five rounds of two builds take about two minutes. Add more `label=binary` pairs to compare more builds.

4. Before reporting a speed-up, confirm the builds still answer identically: start both on spare ports and diff `curl -s -i` output for the same queries, ignoring the `date` header.

WARNING: Builds made without `--target aarch64-apple-darwin` follow the host toolchain; an x86_64 host toolchain yields a binary that runs under Rosetta and benchmarks slower than what the app ships. Pass the target for every build being compared.

WARNING: Port 5050 and `~/ezproxy.txt` belong to the running menu bar app. Leave them alone; the script uses port 5599 and the example config.

WARNING: A cargo build running during the benchmark distorts every number. Finish all builds first.

## Scenarios

| Scenario | Query | Connections |
|---|---|---|
| `static-keepalive-c1` | `m` | 1, kept alive |
| `args-keepalive-c1` | `npm file finder` | 1, kept alive |
| `fallback-keepalive-c1` | `best restaurants nyc` | 1, kept alive |
| `args-keepalive-c32` | `npm file finder` | 32, kept alive |
| `static-newconn-c1` | `m` | a new connection per request |

`static-newconn-c1` is closest to a browser, which reconnects between redirects typed minutes apart. It sends only 1,500 requests per run: each closed connection holds one of macOS's 16,384 ephemeral ports for 30 seconds.

## Reading the summary

Each line is `metric label median [min .. max] change-vs-first-label`.

- `server_instructions_per_redirect`: the steadiest metric, within about 2% between rounds even on a busy machine. Lead with it.
- `server_cpu_us_per_redirect`: CPU time the server spent per redirect. Steady on an idle machine; drifts when the server lands on efficiency cores.
- `redirects_per_second` and `mean_latency_us`: what `ab` saw. `ab` is single-threaded and is usually the bottleneck, so these understate server gains and swing with machine load.

A change smaller than the `[min .. max]` spread of either build is noise. Check `uptime` before running; with a load average above the core count, rely on instructions only or add rounds.

## Finding where the time goes

Start a build on port 5599, keep `ab -k -c 1 -n 400000` running against it, and run `sample <pid> 5 -file /tmp/profile.txt`. Read the "Sort by top of stack" section. `kevent` samples are mostly the server waiting for `ab`, and `__psynch_cvwait` samples are parked worker threads; neither is work.

## Reporting

State the rounds, the machine's load, and for each scenario the baseline and candidate medians with the percentage change for CPU and instructions per redirect. Say that redirects per second is bounded by `ab`.
