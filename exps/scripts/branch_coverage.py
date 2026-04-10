#!/usr/bin/env python3
"""branch_coverage.py — extract and measure branch coverage from llvm-cov export JSON.

Ground truth branch list and coverage measurement based on actual branch entries
in llvm-cov export data. NOT based on llvm-cov report or summary counts.

Each branch entry in export JSON represents one branch condition with true/false
counts. A branch is fully covered when BOTH true > 0 AND false > 0.

Usage:
  # Extract ground truth (all branches, no profile needed):
  llvm-cov-21 export ./bin -empty-profile > export.json
  python3 branch_coverage.py extract export.json branches.json

  # Measure coverage:
  llvm-cov-21 export ./bin -instr-profile=test.profdata > export.json
  python3 branch_coverage.py measure export.json [branches.json]

  # Compare two profiles:
  python3 branch_coverage.py diff export_a.json export_b.json
"""

import json
import sys
import os


def extract_branches_from_file(finfo):
    """Extract all branch entries from a file, including nested expansions."""
    branches = []
    _extract_recursive(finfo, finfo['filename'], branches)
    return branches


def _extract_recursive(obj, source_file, results):
    """Recursively collect branch entries from top-level and expansions."""
    for b in obj.get('branches', []):
        results.append({
            'file': source_file,
            'line': b[0],
            'col': b[1],
            'line_end': b[2],
            'col_end': b[3],
            'true': b[4],
            'false': b[5],
        })
    for exp in obj.get('expansions', []):
        _extract_recursive(exp, source_file, results)


def is_branch_fully_covered(branch):
    """A branch is fully covered when both true and false conditions are exercised."""
    return branch['true'] > 0 and branch['false'] > 0


def count_conditions(branches):
    """Count total and covered branch CONDITIONS (each branch = 2 conditions)."""
    total = len(branches) * 2
    covered = sum((1 if b['true'] > 0 else 0) + (1 if b['false'] > 0 else 0) for b in branches)
    return total, covered


def load_branches(export_json_path, filter_prefix='libm/'):
    """Load all branch entries from llvm-cov export JSON."""
    with open(export_json_path) as f:
        d = json.load(f)

    all_branches = []
    for finfo in d['data'][0]['files']:
        fn = finfo['filename']
        if filter_prefix and filter_prefix not in fn:
            continue
        # Normalize filename to relative path from libm/
        if 'libm/' in fn:
            short = fn[fn.index('libm/'):]
        else:
            short = fn

        for b in extract_branches_from_file(finfo):
            b['file'] = short if 'libm/' in b['file'] else b['file']
            all_branches.append(b)

    return all_branches


def deduplicate(branches):
    """Deduplicate branches by (file, line, col). Keep max counts."""
    seen = {}
    for b in branches:
        key = (b['file'], b['line'], b['col'])
        if key not in seen:
            seen[key] = b.copy()
        else:
            # Keep the max counts (in case same branch appears multiple times)
            seen[key]['true'] = max(seen[key]['true'], b['true'])
            seen[key]['false'] = max(seen[key]['false'], b['false'])
    return list(seen.values())


def cmd_extract(args):
    """Extract ground truth branch list (use with -empty-profile export)."""
    export_path = args[0]
    output_path = args[1] if len(args) > 1 else None

    branches = load_branches(export_path)
    branches = deduplicate(branches)
    branches.sort(key=lambda b: (b['file'], b['line'], b['col']))

    result = {
        'total_branches': len(branches),
        'total_conditions': len(branches) * 2,
        'branches': [
            {'file': b['file'], 'line': b['line'], 'col': b['col']}
            for b in branches
        ]
    }

    if output_path:
        with open(output_path, 'w') as f:
            json.dump(result, f, indent=2)
        print(f"Extracted {len(branches)} branch conditions -> {output_path}")
    else:
        print(json.dumps(result, indent=2))

    # Also print human-readable summary
    per_file = {}
    for b in branches:
        per_file[b['file']] = per_file.get(b['file'], 0) + 1
    print(f"\nPer-file branch counts ({len(per_file)} files):")
    for fn in sorted(per_file.keys()):
        print(f"  {fn}: {per_file[fn]}")
    print(f"\nTotal: {len(branches)}")


def cmd_measure(args):
    """Measure branch condition coverage from a profiled export."""
    export_path = args[0]

    branches = load_branches(export_path)
    branches = deduplicate(branches)

    total_cond, covered_cond = count_conditions(branches)
    uncovered_cond = total_cond - covered_cond

    print(f"Branch condition coverage: {covered_cond}/{total_cond} ({100*covered_cond/total_cond:.1f}%)" if total_cond > 0 else "No branches")
    print(f"  Branches: {len(branches)}")
    print(f"  Conditions (branches × 2): {total_cond}")
    print(f"  Conditions covered (count > 0): {covered_cond}")
    print(f"  Conditions uncovered: {uncovered_cond}")

    # Per-file breakdown
    per_file = {}
    for b in branches:
        fn = b['file']
        if fn not in per_file:
            per_file[fn] = {'branches': 0, 'cond_total': 0, 'cond_covered': 0}
        per_file[fn]['branches'] += 1
        per_file[fn]['cond_total'] += 2
        per_file[fn]['cond_covered'] += (1 if b['true'] > 0 else 0) + (1 if b['false'] > 0 else 0)

    print(f"\nPer-file ({len(per_file)} files):")
    for fn in sorted(per_file.keys()):
        t = per_file[fn]['cond_total']
        c = per_file[fn]['cond_covered']
        pct = 100 * c / t if t > 0 else 0
        if c < t:
            print(f"  {fn}: {c}/{t} conditions ({pct:.0f}%)")


def cmd_uncovered(args):
    """Output uncovered branch conditions (for testgen feedback)."""
    export_path = args[0]
    output_path = args[1] if len(args) > 1 else None

    branches = load_branches(export_path)
    branches = deduplicate(branches)

    total_cond, covered_cond = count_conditions(branches)

    # List each uncovered condition
    uncovered_conditions = []
    for b in branches:
        if b['true'] == 0:
            uncovered_conditions.append({
                **b, 'missing': 'true'
            })
        if b['false'] == 0:
            uncovered_conditions.append({
                **b, 'missing': 'false'
            })

    uncovered_conditions.sort(key=lambda x: (x['file'], x['line'], x['col'], x['missing']))

    lines = []
    lines.append(f"# Uncovered branch conditions")
    lines.append(f"# Total conditions: {total_cond} ({len(branches)} branches × 2)")
    lines.append(f"# Covered: {covered_cond} | Uncovered: {len(uncovered_conditions)}")
    lines.append(f"# Coverage: {100*covered_cond/total_cond:.1f}%" if total_cond > 0 else "# Coverage: N/A")
    lines.append("")
    for c in uncovered_conditions:
        lines.append(f"{c['file']}:{c['line']}:{c['col']}:{c['missing']}")

    text = "\n".join(lines) + "\n"
    if output_path:
        with open(output_path, 'w') as f:
            f.write(text)
        print(f"Uncovered conditions: {len(uncovered_conditions)}/{total_cond} -> {output_path}")
    else:
        print(text)


if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Usage:")
        print("  branch_coverage.py extract <export.json> [output.json]")
        print("  branch_coverage.py measure <export.json>")
        print("  branch_coverage.py uncovered <export.json> [output.md]")
        sys.exit(1)

    cmd = sys.argv[1]
    args = sys.argv[2:]

    if cmd == 'extract':
        cmd_extract(args)
    elif cmd == 'measure':
        cmd_measure(args)
    elif cmd == 'uncovered':
        cmd_uncovered(args)
    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)
