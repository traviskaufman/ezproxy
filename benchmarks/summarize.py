import csv
import statistics
import sys
from collections import defaultdict

METRICS = [
    "redirects_per_second",
    "mean_latency_us",
    "server_cpu_us_per_redirect",
    "server_instructions_per_redirect",
]

with open(sys.argv[1]) as results:
    rows = list(csv.DictReader(results))

rows_with_failures = [row for row in rows if row["failed_requests"] != "0"]
if rows_with_failures:
    sys.exit(f"requests failed in: {rows_with_failures}")

samples = defaultdict(list)
for row in rows:
    for metric in METRICS:
        samples[row["scenario"], row["label"], metric].append(float(row[metric]))

scenarios = list(dict.fromkeys(row["scenario"] for row in rows))
labels = list(dict.fromkeys(row["label"] for row in rows))
baseline = labels[0]

for scenario in scenarios:
    print(scenario)
    for metric in METRICS:
        baseline_median = statistics.median(samples[scenario, baseline, metric])
        for label in labels:
            values = samples[scenario, label, metric]
            median = statistics.median(values)
            change = 100 * (median - baseline_median) / baseline_median
            print(
                f"  {metric:34s} {label:12s} {median:12.2f} "
                f"[{min(values):.2f} .. {max(values):.2f}] {change:+6.1f}%"
            )
