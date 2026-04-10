#!/usr/bin/env python3
"""Build a minimal context file for difffix analysis.

Given a diff report (C output vs Rust output), extract:
1. Which functions diverged
2. For each diverging function: the C source and Rust source (just that function)

This gives the AI exactly what it needs to diagnose and fix — nothing more.
Typically ~500-2000 tokens per diverging function instead of ~170K for the whole lib.

Usage:
    python3 make_difffix_context.py <c_output> <rust_output> <rust_src_dir> [--max-funcs N]

Output: printed to stdout, pipe to a file for the AI.
"""

import os
import re
import sys
import argparse


def parse_diff(c_lines, r_lines):
    """Find lines that differ, extract function names."""
    divergences = {}  # func_name -> [(c_line, r_line)]
    for c, r in zip(c_lines, r_lines):
        c = c.strip()
        r = r.strip()
        if c != r:
            # Lines are like "T001 sin 0x1.acd2p+0"
            parts = c.split()
            if len(parts) >= 2:
                func = parts[1] if len(parts) > 1 else parts[0]
                if func not in divergences:
                    divergences[func] = []
                divergences[func].append((c, r))
    return divergences


def find_c_source(func_name, libm_dir):
    """Find the C source file for a function."""
    # Convention: sin -> sind.c or sinf.c, etc.
    candidates = [
        f"{func_name}.c",
        f"{func_name}d.c",   # double: sin -> sind.c
        f"{func_name}f.c",   # float: sinf -> sinff.c? No...
    ]
    # Also try the name as-is in each directory
    for src_dir in ["mathd", "mathf", "common", "complexd", "complexf"]:
        full_dir = os.path.join(libm_dir, src_dir)
        if not os.path.isdir(full_dir):
            continue
        for fname in os.listdir(full_dir):
            if not fname.endswith('.c'):
                continue
            # Check if function is defined in this file
            path = os.path.join(full_dir, fname)
            with open(path) as f:
                content = f.read()
            # Look for function definition
            if re.search(rf'\b{re.escape(func_name)}\s*\(', content):
                return path, content
    return None, None


def find_rust_function(func_name, rust_src_dir):
    """Find a function in the Rust source files."""
    for root, dirs, files in os.walk(rust_src_dir):
        for fname in files:
            if not fname.endswith('.rs'):
                continue
            path = os.path.join(root, fname)
            with open(path) as f:
                content = f.read()
            # Look for pub fn func_name or fn func_name
            pattern = rf'((?:pub\s+)?fn\s+{re.escape(func_name)}\s*\([^)]*\)[^{{]*\{{)'
            match = re.search(pattern, content)
            if match:
                # Extract the full function body (brace matching)
                start = match.start()
                depth = 0
                i = content.index('{', start)
                for j in range(i, len(content)):
                    if content[j] == '{':
                        depth += 1
                    elif content[j] == '}':
                        depth -= 1
                        if depth == 0:
                            return path, content[start:j+1]
                return path, content[start:]
    return None, None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("c_output", help="C test output file")
    parser.add_argument("rust_output", help="Rust test output file")
    parser.add_argument("rust_src_dir", help="Rust source directory")
    parser.add_argument("--libm", default="")
    parser.add_argument("--max-funcs", type=int, default=10)
    args = parser.parse_args()

    with open(args.c_output) as f:
        c_lines = f.readlines()
    with open(args.rust_output) as f:
        r_lines = f.readlines()

    divergences = parse_diff(c_lines, r_lines)

    if not divergences:
        print("No divergences found.")
        return

    print(f"# Divergence Report: {len(divergences)} functions differ\n")

    for i, (func, diffs) in enumerate(sorted(divergences.items(),
                                              key=lambda x: -len(x[1]))):
        if i >= args.max_funcs:
            print(f"\n... and {len(divergences) - args.max_funcs} more functions")
            break

        print(f"## {func} — {len(diffs)} divergences\n")

        # Show first few divergences
        print("Sample divergences:")
        for c, r in diffs[:5]:
            print(f"  C:    {c}")
            print(f"  Rust: {r}")
        if len(diffs) > 5:
            print(f"  ... ({len(diffs) - 5} more)")
        print()

        # C source
        c_path, c_content = find_c_source(func, args.libm)
        if c_path:
            print(f"### C source: {c_path}")
            print(f"```c\n{c_content}\n```\n")
        else:
            print(f"### C source: not found for '{func}'\n")

        # Rust source
        r_path, r_content = find_rust_function(func, args.rust_src_dir)
        if r_path:
            print(f"### Rust source: {r_path}")
            print(f"```rust\n{r_content}\n```\n")
        else:
            print(f"### Rust source: not found for '{func}'\n")


if __name__ == "__main__":
    main()
