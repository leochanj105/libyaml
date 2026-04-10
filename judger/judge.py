#!/usr/bin/env python3
"""
judge.py — End-to-end differential testing pipeline.

  1. Run all test cases against library A   → A/results.jsonl
  2. Run all test cases against library B   → B/results.jsonl
  3. Compare A vs B case-by-case            → match / diff / panic counts

Usage:
    judge.py --lib-a=PATH --lib-b=PATH --outdir=DIR [--timeout=SEC]
"""

from __future__ import annotations
import argparse
import subprocess
import sys
from pathlib import Path

JUDGER_DIR = Path(__file__).resolve().parent


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--lib-a", required=True, help="Path to library A (.a)")
    p.add_argument("--lib-b", required=True, help="Path to library B (.a)")
    p.add_argument("--outdir", required=True, help="Output directory")
    p.add_argument("--timeout", type=int, default=10)
    p.add_argument("--include", default=str(JUDGER_DIR.parent / "include"))
    p.add_argument("--src", default=str(JUDGER_DIR.parent / "src"))
    args = p.parse_args()

    outdir = Path(args.outdir).resolve()
    outdir.mkdir(parents=True, exist_ok=True)

    dir_a = outdir / "A"
    dir_b = outdir / "B"

    print(f"[judge] Library A: {args.lib_a}")
    print(f"[judge] Library B: {args.lib_b}")
    print(f"[judge] Output:    {outdir}")
    print()

    # Step 1+2: Run against each library
    for label, lib, dir_ in [("A", args.lib_a, dir_a), ("B", args.lib_b, dir_b)]:
        print(f"[judge] === Running against library {label} ===")
        rc = subprocess.run([
            sys.executable, str(JUDGER_DIR / "run.py"),
            f"--lib={lib}",
            f"--outdir={dir_}",
            f"--timeout={args.timeout}",
            f"--include={args.include}",
            f"--src={args.src}",
        ]).returncode
        if rc != 0:
            print(f"[judge] FAILED to run against library {label}", file=sys.stderr)
            return rc
        print()

    # Step 3: Compare
    print(f"[judge] === Comparing A vs B ===")
    rc = subprocess.run([
        sys.executable, str(JUDGER_DIR / "compare.py"),
        f"--a={dir_a}",
        f"--b={dir_b}",
        f"--report={outdir}/report.tsv",
    ]).returncode

    return rc


if __name__ == "__main__":
    sys.exit(main())
