#!/usr/bin/env python3
"""
compare.py — Compare two judger result files (results.jsonl from run.py).

For each test case in A, find the matching case in B (by function+input) and
classify the comparison:

  match:  exit_code, stdout, stderr all bitwise identical
  diff:   outputs differ, but neither side timed out or crashed
  panic:  one or both sides timed out or were killed by a signal

Usage:
    compare.py --a=DIR_A --b=DIR_B [--report=PATH]

Output:
    Summary printed to stdout.
    Optional --report=PATH writes a per-case TSV with classifications.
"""

from __future__ import annotations
import argparse
import json
import sys
from collections import Counter
from pathlib import Path


def load_results(path: Path) -> dict[tuple[str, str], dict]:
    """Load a results.jsonl file. Returns a dict keyed by (function, input)."""
    results = {}
    with path.open() as f:
        for line in f:
            r = json.loads(line)
            key = (r["function"], r["input"])
            if key in results:
                print(f"WARNING: duplicate key {key} in {path}", file=sys.stderr)
            results[key] = r
    return results


def classify(a: dict, b: dict) -> str:
    """Classify a comparison: 'match', 'diff', or 'panic'."""
    same = (
        a["exit_code"] == b["exit_code"]
        and a["stdout"] == b["stdout"]
        and a["stderr"] == b["stderr"]
    )
    if same:
        return "match"
    # Differs — was it a panic on either side?
    a_panic = a["timeout"] or a["signal"] is not None
    b_panic = b["timeout"] or b["signal"] is not None
    if a_panic or b_panic:
        return "panic"
    return "diff"


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--a", required=True, help="Result directory A")
    p.add_argument("--b", required=True, help="Result directory B")
    p.add_argument("--report", help="Optional TSV report path")
    args = p.parse_args()

    file_a = Path(args.a) / "results.jsonl"
    file_b = Path(args.b) / "results.jsonl"

    if not file_a.is_file():
        print(f"Error: {file_a} not found", file=sys.stderr); return 1
    if not file_b.is_file():
        print(f"Error: {file_b} not found", file=sys.stderr); return 1

    results_a = load_results(file_a)
    results_b = load_results(file_b)

    keys_a = set(results_a.keys())
    keys_b = set(results_b.keys())
    only_a = keys_a - keys_b
    only_b = keys_b - keys_a
    common = sorted(keys_a & keys_b)

    counts = Counter()
    rows = []
    for key in common:
        verdict = classify(results_a[key], results_b[key])
        counts[verdict] += 1
        rows.append((verdict, key[0], key[1]))

    # Print summary
    total = len(common)
    print(f"=== Comparison: {args.a} vs {args.b} ===")
    print(f"Total cases (in both):  {total}")
    print(f"  match:  {counts['match']}")
    print(f"  diff:   {counts['diff']}")
    print(f"  panic:  {counts['panic']}")
    if only_a:
        print(f"  only in A: {len(only_a)}")
    if only_b:
        print(f"  only in B: {len(only_b)}")

    # First few failures
    failures = [r for r in rows if r[0] != "match"]
    if failures:
        print()
        print(f"First 10 failures:")
        for verdict, func, inp in failures[:10]:
            print(f"  [{verdict:5}] {func}  {inp}")
        if len(failures) > 10:
            print(f"  ... and {len(failures) - 10} more")

    # Write TSV report
    if args.report:
        with open(args.report, "w") as f:
            f.write("verdict\tfunction\tinput\n")
            for verdict, func, inp in rows:
                f.write(f"{verdict}\t{func}\t{inp}\n")
        print(f"\nFull report: {args.report}")

    # Exit non-zero if any failures
    return 0 if (counts["diff"] == 0 and counts["panic"] == 0) else 1


if __name__ == "__main__":
    sys.exit(main())
