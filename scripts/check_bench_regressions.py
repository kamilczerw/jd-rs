#!/usr/bin/env python3
"""Compare Criterion benchmark results against a stored baseline."""
from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path


def load_estimate(path: Path) -> float:
    data = json.loads(path.read_text())
    try:
        return float(data["median"]["point_estimate"])
    except (KeyError, TypeError, ValueError) as exc:
        raise SystemExit(f"invalid estimates file {path}: {exc}")


def compare(baseline_path: Path, results_root: Path, run_name: str, tolerance: float) -> int:
    baseline = json.loads(baseline_path.read_text())
    metadata = baseline.get("metadata", {})
    baseline_tolerance = metadata.get("tolerance")
    tol = tolerance if tolerance is not None else baseline_tolerance
    if tol is None:
        tol = 1.25
    benchmarks = baseline.get("benchmarks", {})
    failures: list[str] = []
    for group, entries in sorted(benchmarks.items()):
        for bench, target in sorted(entries.items()):
            estimates_path = results_root / group / bench / run_name / "estimates.json"
            if not estimates_path.is_file():
                failures.append(f"missing results for {group}/{bench}: {estimates_path} not found")
                continue
            actual = load_estimate(estimates_path)
            ratio = actual / float(target)
            if math.isnan(ratio) or math.isinf(ratio):
                failures.append(f"invalid ratio for {group}/{bench}: actual={actual} baseline={target}")
                continue
            if ratio > tol:
                failures.append(
                    f"regression detected for {group}/{bench}: actual {actual:.3f} ns exceeds baseline {target:.3f} ns by {ratio:.2f}x (limit {tol:.2f}x)"
                )
            else:
                print(
                    f"ok {group}/{bench}: actual {actual:.3f} ns vs baseline {target:.3f} ns ({ratio:.2f}x)"
                )
    if failures:
        for failure in failures:
            print(f"::error ::{failure}")
        return 1
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--baseline",
        type=Path,
        default=Path("crates/jd-benches/baselines/criterion-ci.json"),
        help="Path to the baseline JSON file.",
    )
    parser.add_argument(
        "--results-root",
        type=Path,
        default=Path("target/criterion"),
        help="Directory containing Criterion output.",
    )
    parser.add_argument(
        "--run-name",
        default="current",
        help="Criterion baseline/run directory name to compare (default: current).",
    )
    parser.add_argument(
        "--tolerance",
        type=float,
        default=None,
        help="Override tolerance multiplier (default uses baseline metadata).",
    )
    args = parser.parse_args(argv)
    return compare(args.baseline, args.results_root, args.run_name, args.tolerance)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
