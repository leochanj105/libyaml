"""List all branch points in a coverage-instrumented binary.

Each branch is one binary decision point (IR-level conditional). With -empty-profile
we get every branch from the binary's coverage map; with a real profile we get
the same list plus per-edge execution counts.

Output format (default): <file>:<line>
With --by-file: per-file counts only.
With --with-cols: <file>:<line>:<col>

Usage:
    python3 -m cov.list_branches <binary> [--by-file]
                                          [--exclude-headers]
                                          [--file FILENAME]
                                          [--profdata PATH]
"""
import argparse
import sys

from .loader import llvm_cov_export, basename


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("binary")
    p.add_argument("--profdata", help="If set, real counts are read (default: empty)")
    p.add_argument("--by-file", action="store_true", help="Print per-file counts only")
    p.add_argument("--exclude-headers", action="store_true",
                   help="Skip .h files (e.g. yaml_private.h macro entries)")
    p.add_argument("--file", dest="file_filter",
                   help="Show branches from one file only (e.g. scanner.c)")
    p.add_argument("--with-cols", action="store_true",
                   help="Include column numbers in output")
    p.add_argument("--llvm-cov", default="llvm-cov-21")
    args = p.parse_args(argv)

    data = llvm_cov_export(args.binary, args.profdata, args.llvm_cov)
    files = data["data"][0]["files"]

    per_file = {}
    flat = []
    for f in files:
        fname = basename(f["filename"])
        if args.exclude_headers and fname.endswith(".h"):
            continue
        if args.file_filter and fname != args.file_filter:
            continue
        branches = f.get("branches", [])
        per_file[fname] = len(branches)
        for b in branches:
            l1, c1, l2, c2 = b[0], b[1], b[2], b[3]
            flat.append((fname, l1, c1, l2, c2))

    if args.by_file:
        total = 0
        for fname in sorted(per_file):
            print(f"{fname:20s} {per_file[fname]}")
            total += per_file[fname]
        print(f"{'TOTAL':20s} {total}")
    else:
        flat.sort()
        for fname, l1, c1, l2, c2 in flat:
            if args.with_cols:
                print(f"{fname}:{l1}:{c1}")
            else:
                print(f"{fname}:{l1}")


if __name__ == "__main__":
    main()
