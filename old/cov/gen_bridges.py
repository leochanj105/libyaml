"""Generate test bridge wrappers for static functions in libyaml.

For each .c file in src/, produce a bridge_<name>.c file under build_cov/bridges/
that:
  1. #includes the original .c source (so it has access to static functions)
  2. Defines bridge_<funcname>(...) wrappers calling each static function

The bridges are linked into the test binary so test code can call
bridge_<funcname>(...) instead of the unreachable static <funcname>(...).

Linker --allow-multiple-definition is required because each bridge .c file
re-#includes a source .c file, redefining all of its non-static symbols.

Usage:
    python3 -m cov.gen_bridges <src_dir> <out_dir>
"""
import argparse
import os
import re
import subprocess
import sys


# Locate the start of any function definition: '<ret> <name>('.
# We deliberately don't anchor on `static` because clang -ast-print sometimes
# strips the storage class from the definition when it was declared earlier.
# The caller filters by an externally-supplied static-name set.
# `\*?` allows pointer return types with no space (e.g. 'yaml_char_t *foo').
_FUNC_START_RE = re.compile(
    r"^(?:static\s+)?(?P<ret>[\w\s\*]+?[\w\*])\s*(?P<name>\w+)\s*\(",
    re.MULTILINE,
)


def _extract_balanced_args(text, start_idx):
    """Given text and the index of the opening '(', return (args_str, end_idx)
    where end_idx is the index just past the matching ')'."""
    assert text[start_idx] == "("
    depth = 0
    i = start_idx
    while i < len(text):
        c = text[i]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return text[start_idx + 1:i], i + 1
        i += 1
    return None, len(text)


def get_ast_print(c_file, include_dirs, src_dir):
    """Run clang -Xclang -ast-print and return the printed source."""
    cmd = [
        "clang-21", "-Xclang", "-ast-print", "-fsyntax-only",
        "-DHAVE_CONFIG_H=1",
    ]
    for d in include_dirs:
        cmd += ["-I", d]
    cmd += ["-I", src_dir, c_file]
    return subprocess.check_output(cmd, stderr=subprocess.DEVNULL, text=True)


def parse_function_defs(ast_text, name_filter=None):
    """Return list of (name, return_type, arg_decls, arg_names) for function
    DEFINITIONS (not declarations). A definition has '{' after ')'.

    If name_filter is given (a set), only functions whose name is in the set
    are returned. Otherwise all definitions are returned.
    """
    funcs = []
    seen = set()
    for m in _FUNC_START_RE.finditer(ast_text):
        ret = m.group("ret").strip()
        name = m.group("name").strip()
        if name_filter is not None and name not in name_filter:
            continue
        paren_idx = m.end() - 1
        args, end_idx = _extract_balanced_args(ast_text, paren_idx)
        if args is None:
            continue
        rest = ast_text[end_idx:end_idx + 200].lstrip()
        if not rest.startswith("{"):
            continue  # forward declaration only
        if name in seen:
            continue
        seen.add(name)

        # Normalize pointer return types: 'yaml_char_t *' -> the '*' may have
        # been captured at the end of `ret` (e.g. 'yaml_char_t' or 'yaml_char_t *').
        ret = ret.strip()
        args = args.strip()
        if args == "" or args == "void":
            arg_decls = "void"
            arg_names = []
        else:
            arg_decls = args
            arg_names = []
            for param in _split_params(args):
                arg_names.append(_extract_param_name(param))
        funcs.append((name, ret, arg_decls, arg_names))
    return funcs


def get_static_function_names(binary, llvm_cov="llvm-cov-21"):
    """Use llvm-cov export -empty-profile on a coverage-instrumented binary
    to get the authoritative list of static functions, grouped by source file."""
    import json
    out = subprocess.check_output(
        [llvm_cov, "export", binary, "-empty-profile"],
        stderr=subprocess.DEVNULL,
    )
    data = json.loads(out)
    by_file = {}
    for f in data["data"][0]["functions"]:
        name = f["name"]
        if ":" not in name:
            continue  # not a static function
        bare = name.split(":", 1)[-1]
        src = f["filenames"][0].rsplit("/", 1)[-1]
        by_file.setdefault(src, set()).add(bare)
    return by_file


def _extract_param_name(param):
    """Extract the parameter name, ignoring __attribute__((...)) suffixes."""
    # Strip trailing attributes
    cleaned = re.sub(r"__attribute__\s*\(\([^)]*\)\)", "", param).strip()
    # Strip array brackets
    cleaned = re.sub(r"\[[^\]]*\]\s*$", "", cleaned).strip()
    # Take the last word — that's the parameter name
    m = re.search(r"(\w+)\s*$", cleaned)
    return m.group(1) if m else ""


def _split_params(args):
    """Split parameter list on commas, respecting nested parens (for function pointers)."""
    out = []
    depth = 0
    cur = []
    for ch in args:
        if ch == "(":
            depth += 1
            cur.append(ch)
        elif ch == ")":
            depth -= 1
            cur.append(ch)
        elif ch == "," and depth == 0:
            out.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    if cur:
        out.append("".join(cur))
    return out


def emit_bridge(c_file, out_path, funcs):
    """Write a bridge .c file that #includes c_file and exports bridge_* wrappers."""
    abs_src = os.path.abspath(c_file)
    lines = []
    lines.append("/* AUTO-GENERATED by cov/gen_bridges.py — do not edit. */")
    lines.append(f'#include "{abs_src}"')
    lines.append("")
    for name, ret, arg_decls, arg_names in funcs:
        # Wrapper signature: bridge_<name>(<args>) — non-static, callable externally
        call_args = ", ".join(arg_names) if arg_names else ""
        ret_clean = ret.strip()
        is_void = ret_clean == "void"
        lines.append(f"{ret_clean} bridge_{name}({arg_decls}) {{")
        if is_void:
            lines.append(f"    {name}({call_args});")
        else:
            lines.append(f"    return {name}({call_args});")
        lines.append("}")
        lines.append("")
    with open(out_path, "w") as f:
        f.write("\n".join(lines))


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("src_dir", help="Path to libyaml src/ directory")
    p.add_argument("out_dir", help="Output directory for bridge_*.c files")
    p.add_argument("--binary", required=True,
                   help="Coverage-instrumented binary used to get the authoritative "
                        "static function list (e.g. build_cov/test_bin)")
    p.add_argument("--include", "-I", action="append", default=[],
                   help="Additional include directories")
    p.add_argument("--llvm-cov", default="llvm-cov-21")
    args = p.parse_args(argv)

    include_dirs = list(args.include)
    libyaml_root = os.path.dirname(os.path.abspath(args.src_dir))
    for d in (
        os.path.join(libyaml_root, "include"),
        os.path.join(libyaml_root, "build", "include"),
    ):
        if os.path.isdir(d):
            include_dirs.append(d)

    os.makedirs(args.out_dir, exist_ok=True)

    static_by_file = get_static_function_names(args.binary, args.llvm_cov)
    total_expected = sum(len(s) for s in static_by_file.values())
    print(f"Static functions to bridge (from {args.binary}): {total_expected}")
    print()

    total = 0
    for fname in sorted(os.listdir(args.src_dir)):
        if not fname.endswith(".c"):
            continue
        wanted = static_by_file.get(fname, set())
        if not wanted:
            continue
        c_file = os.path.join(args.src_dir, fname)
        try:
            ast = get_ast_print(c_file, include_dirs, args.src_dir)
        except subprocess.CalledProcessError as e:
            print(f"  WARNING: clang -ast-print failed on {fname}: {e}", file=sys.stderr)
            continue
        funcs = parse_function_defs(ast, name_filter=wanted)
        found_names = {f[0] for f in funcs}
        missing = wanted - found_names
        out_path = os.path.join(args.out_dir, f"bridge_{fname}")
        emit_bridge(c_file, out_path, funcs)
        msg = f"  {fname}: {len(funcs)}/{len(wanted)} bridge wrappers → {out_path}"
        if missing:
            msg += f"  (MISSING: {', '.join(sorted(missing))})"
        print(msg)
        total += len(funcs)

    print()
    print(f"Total bridge wrappers generated: {total}/{total_expected}")


if __name__ == "__main__":
    main()
