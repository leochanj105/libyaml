"""List all functions in a coverage-instrumented binary.

Static functions appear as `file.c:funcname`; non-static as just `funcname`.

Usage:
    python3 -m cov.list_functions <binary> [--static-only|--public-only]
                                          [--profdata PATH]   # only show executed
                                          [--uncovered]       # only show NOT executed
"""
import argparse
import sys

from .loader import llvm_cov_export, basename


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("binary", help="Path to instrumented binary")
    p.add_argument("--profdata", help="If set, filter by execution count")
    g = p.add_mutually_exclusive_group()
    g.add_argument("--static-only", action="store_true")
    g.add_argument("--public-only", action="store_true")
    p.add_argument("--uncovered", action="store_true",
                   help="With --profdata, only show functions with count == 0")
    p.add_argument("--executed", action="store_true",
                   help="With --profdata, only show functions with count > 0")
    p.add_argument("--llvm-cov", default="llvm-cov-21")
    args = p.parse_args(argv)

    data = llvm_cov_export(args.binary, args.profdata, args.llvm_cov)
    funcs = data["data"][0]["functions"]

    out = []
    for f in sorted(funcs, key=lambda x: (x["filenames"][0], x["name"])):
        name = f["name"]
        is_static = ":" in name
        if args.static_only and not is_static:
            continue
        if args.public_only and is_static:
            continue
        if args.uncovered and f["count"] != 0:
            continue
        if args.executed and f["count"] == 0:
            continue
        bare = name.split(":", 1)[-1] if is_static else name
        out.append(bare)

    for n in out:
        print(n)


if __name__ == "__main__":
    main()
