"""List branch conditions NOT covered in a test run.

Each branch has 2 conditions (true edge + false edge). A condition is "covered"
if its execution count > 0.

Output format (default): <file>:<line>:<col>:<true|false>
With --summary: per-file totals on stderr only.

Usage:
    python3 -m cov.list_uncovered_branches <binary> --profdata <profdata>
                                                    [--exclude-headers]
                                                    [--file FILENAME]
                                                    [--summary]
"""
import argparse
import sys
from collections import defaultdict

from .loader import llvm_cov_export, basename


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("binary")
    p.add_argument("--profdata", required=True,
                   help="Path to merged .profdata from a test run")
    p.add_argument("--exclude-headers", action="store_true")
    p.add_argument("--file", dest="file_filter")
    p.add_argument("--summary", action="store_true",
                   help="Print only per-file summary on stderr, no detail list")
    p.add_argument("--llvm-cov", default="llvm-cov-21")
    args = p.parse_args(argv)

    data = llvm_cov_export(args.binary, args.profdata, args.llvm_cov)
    files = data["data"][0]["files"]

    per_file_total = defaultdict(int)
    per_file_uncov = defaultdict(int)
    uncovered = []

    for f in files:
        fname = basename(f["filename"])
        if args.exclude_headers and fname.endswith(".h"):
            continue
        if args.file_filter and fname != args.file_filter:
            continue
        for b in f.get("branches", []):
            l1, c1, t, fcount = b[0], b[1], b[4], b[5]
            per_file_total[fname] += 2
            if t == 0:
                per_file_uncov[fname] += 1
                uncovered.append((fname, l1, c1, "true"))
            if fcount == 0:
                per_file_uncov[fname] += 1
                uncovered.append((fname, l1, c1, "false"))

    total_cond = sum(per_file_total.values())
    total_uncov = sum(per_file_uncov.values())
    total_cov = total_cond - total_uncov

    if not args.summary:
        uncovered.sort()
        for fname, l, c, miss in uncovered:
            print(f"{fname}:{l}:{c}:{miss}")

    print("# ─────────────────────────────────", file=sys.stderr)
    print("# Per file:", file=sys.stderr)
    for fname in sorted(per_file_total):
        t = per_file_total[fname]
        u = per_file_uncov[fname]
        pct = 100 * (t - u) / t if t > 0 else 0
        print(f"#   {fname:20s} {t-u:5d}/{t:5d} covered ({pct:5.1f}%)  {u:5d} uncovered",
              file=sys.stderr)
    pct = 100 * total_cov / total_cond if total_cond > 0 else 0
    print(f"# TOTAL: {total_cov}/{total_cond} conditions covered ({pct:.1f}%), "
          f"{total_uncov} uncovered", file=sys.stderr)


if __name__ == "__main__":
    main()
