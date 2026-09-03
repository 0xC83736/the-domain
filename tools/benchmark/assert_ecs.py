#!/usr/bin/env python3
"""
assert_ecs.py — Phase 1 CI gate for ECS performance targets.

Runs the nexus_ecs criterion benchmarks via `cargo bench --bench ecs_bench`
and parses the output to assert against hard thresholds.

Usage:
    python tools/benchmark/assert_ecs.py --max-spawn-1m-ms 50 --max-query-100k-ms 2
"""

import argparse
import subprocess
import re
import sys


def parse_criterion_time(line: str) -> float | None:
    """Extract median time in ms from a criterion output line."""
    match = re.search(r'time:\s+\[.*?([\d.]+)\s+(ms|µs|ns)', line)
    if not match:
        return None
    value, unit = float(match.group(1)), match.group(2)
    if unit == 'µs':
        value /= 1000
    elif unit == 'ns':
        value /= 1_000_000
    return value


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--max-spawn-1m-ms',   type=float, required=True)
    parser.add_argument('--max-query-100k-ms', type=float, required=True)
    args = parser.parse_args()

    print("Running ECS benchmarks...")
    result = subprocess.run(
        ['cargo', 'bench', '--bench', 'ecs_bench', '--', '--output-format', 'bencher'],
        capture_output=True, text=True
    )

    output = result.stdout + result.stderr
    failures = []

    for line in output.splitlines():
        if 'spawn_despawn_1m' in line:
            t = parse_criterion_time(line)
            if t is not None and t > args.max_spawn_1m_ms:
                failures.append(f"FAIL spawn_despawn_1m: {t:.2f}ms > {args.max_spawn_1m_ms}ms")
            elif t is not None:
                print(f"PASS spawn_despawn_1m: {t:.2f}ms")

        if 'spawn_100k_entity_count' in line:
            t = parse_criterion_time(line)
            if t is not None and t > args.max_query_100k_ms:
                failures.append(f"FAIL query_100k: {t:.4f}ms > {args.max_query_100k_ms}ms")
            elif t is not None:
                print(f"PASS query_100k: {t:.4f}ms")

    if failures:
        print("\n=== BENCHMARK REGRESSION GATE FAILED ===")
        for f in failures:
            print(f)
        sys.exit(1)

    print("\nAll ECS benchmark gates passed.")


if __name__ == '__main__':
    main()
