#!/usr/bin/env python3
"""Fail if any benchmark in `pr` regressed beyond `--threshold` percent vs `base`.

critcmp's own `--threshold` flag only hides small diffs from its printed
table — it isn't a pass/fail gate, and critcmp exits non-zero whenever
there's nothing left to show after filtering, i.e. exactly when there is
*no* regression. This script reads critcmp's `--export` JSON for both
baselines directly and does the actual regression check.
"""

import argparse
import json
import subprocess
import sys


def export_baseline(name: str) -> dict:
    out = subprocess.run(
        ["critcmp", "--export", name], capture_output=True, text=True, check=True
    )
    return json.loads(out.stdout)


def mean(benchmark: dict) -> float:
    return benchmark["criterion_estimates_v1"]["mean"]["point_estimate"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base")
    parser.add_argument("pr")
    parser.add_argument("--threshold", type=float, default=10.0)
    args = parser.parse_args()

    base = export_baseline(args.base)["benchmarks"]
    pr = export_baseline(args.pr)["benchmarks"]

    regressions = []
    for bench_id, pr_bench in pr.items():
        base_bench = base.get(bench_id)
        if base_bench is None:
            continue
        base_mean, pr_mean = mean(base_bench), mean(pr_bench)
        pct_change = (pr_mean - base_mean) / base_mean * 100.0
        marker = "REGRESSION" if pct_change > args.threshold else "ok"
        print(f"{bench_id}: {base_mean:.4f} -> {pr_mean:.4f} ({pct_change:+.2f}%) [{marker}]")
        if pct_change > args.threshold:
            regressions.append((bench_id, pct_change))

    if regressions:
        print(f"\n{len(regressions)} benchmark(s) regressed beyond {args.threshold}%:")
        for bench_id, pct_change in regressions:
            print(f"  - {bench_id}: {pct_change:+.2f}%")
        return 1

    print(f"\nNo benchmark regressed beyond {args.threshold}%.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
