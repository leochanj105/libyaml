#!/usr/bin/env python3
"""
run.py — Run all judger test cases against ONE linked libyaml library.

A test case = one (test_function, input) pair, run as an isolated subprocess
with a timeout. Results are saved as JSON Lines, one record per case.

Usage:
    run.py --lib=PATH --outdir=DIR [--timeout=SEC] [--include=DIR] [--src=DIR]

Output:
    OUTDIR/results.jsonl   one JSON record per test case
    OUTDIR/compile.log     compilation messages
"""

from __future__ import annotations
import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional

JUDGER_DIR = Path(__file__).resolve().parent

# ─────────────────────────────────────────────────────────────────────────────
# Test function inventory
# ─────────────────────────────────────────────────────────────────────────────
# Each test function is a C program in test-functions/. They differ in how
# they receive input and which libyaml APIs they call.

# Take YAML file as argv[1]
ARG_FUNCS = [
    "run-scanner", "run-parser", "run-loader",
    "run-emitter", "run-dumper", "run-parser-test-suite",
]

# Read YAML from stdin
STDIN_FUNCS = [
    "example-reformatter", "example-reformatter-alt",
    "example-deconstructor", "example-deconstructor-alt",
]

# Take test-suite event file as argv[1]
EVENT_FUNCS = ["run-emitter-test-suite"]

# Self-contained: hardcoded test data, no input file
SELF_FUNCS = ["test-version", "test-reader"]

ALL_FUNCS = ARG_FUNCS + STDIN_FUNCS + EVENT_FUNCS + SELF_FUNCS


# ─────────────────────────────────────────────────────────────────────────────
# Data model
# ─────────────────────────────────────────────────────────────────────────────
@dataclass
class TestCase:
    """One test case = one test function × one input."""
    function: str       # e.g. "run-scanner"
    input: str          # e.g. "yaml-test-suite/229Q" or "<self-contained>"
    input_path: Optional[str]  # absolute path to input file, or None
    input_mode: str     # "arg", "stdin", "event", "self"


@dataclass
class TestResult:
    """The result of running one test case."""
    function: str
    input: str
    exit_code: int      # 0-127: normal; 128+N: signal N; 124: timeout
    timeout: bool       # true if killed by timeout
    signal: Optional[int]  # signal number if killed by signal, else None
    stdout: str         # captured stdout (paths normalized)
    stderr: str         # captured stderr (paths normalized)


# ─────────────────────────────────────────────────────────────────────────────
# Test case enumeration
# ─────────────────────────────────────────────────────────────────────────────
def enumerate_test_cases() -> list[TestCase]:
    """Build the list of (function, input) test cases.

    Inputs:
      - For each in.yaml under yaml-test-suite/ (recursively, excluding name/
        and tags/ metadata dirs): pair with each ARG_FUNC and each STDIN_FUNC.
      - For each test.event next to an in.yaml: pair with each EVENT_FUNC.
      - For SELF_FUNCS: one case each, no input.
    """
    cases: list[TestCase] = []
    suite_root = JUDGER_DIR / "yaml-test-suite"

    # Find all in.yaml files, sorted for determinism
    in_yamls = sorted(
        p for p in suite_root.rglob("in.yaml")
        if "name" not in p.parts and "tags" not in p.parts
    )

    for in_yaml in in_yamls:
        rel = in_yaml.parent.relative_to(suite_root)
        input_id = f"yaml-test-suite/{rel}"

        for func in ARG_FUNCS:
            cases.append(TestCase(func, input_id, str(in_yaml), "arg"))
        for func in STDIN_FUNCS:
            cases.append(TestCase(func, input_id, str(in_yaml), "stdin"))

        test_event = in_yaml.parent / "test.event"
        if test_event.exists():
            for func in EVENT_FUNCS:
                cases.append(TestCase(func, input_id, str(test_event), "event"))

    for func in SELF_FUNCS:
        cases.append(TestCase(func, "<self-contained>", None, "self"))

    return cases


# ─────────────────────────────────────────────────────────────────────────────
# Compilation
# ─────────────────────────────────────────────────────────────────────────────
def compile_test_functions(
    bindir: Path, lib_path: Path, include_dir: Path, src_dir: Path,
    cc: str = "cc", cflags: list[str] = (),
) -> dict[str, Path]:
    """Compile each test function .c into a binary linked against lib_path.

    Returns a dict mapping function name → binary path.
    Functions that fail to compile are omitted (with a warning).
    """
    bindir.mkdir(parents=True, exist_ok=True)
    binaries: dict[str, Path] = {}
    log_lines: list[str] = []

    for func in ALL_FUNCS:
        src = JUDGER_DIR / "test-functions" / f"{func}.c"
        binary = bindir / func

        cmd = [cc, *cflags, f"-I{include_dir}"]
        # run-emitter-test-suite needs a private header from src/
        if func == "run-emitter-test-suite":
            cmd.append(f"-I{src_dir.parent}")
        cmd += ["-o", str(binary), str(src), str(lib_path), "-lm"]

        result = subprocess.run(cmd, capture_output=True, text=True)
        log_lines.append(f"=== {func} ===\n{' '.join(cmd)}\n{result.stdout}{result.stderr}\n")

        if result.returncode == 0:
            binaries[func] = binary
        else:
            print(f"[run] WARNING: failed to compile {func}", file=sys.stderr)

    (bindir / "compile.log").write_text("".join(log_lines))
    return binaries


# ─────────────────────────────────────────────────────────────────────────────
# Path normalization (so outputs are reproducible across machines)
# ─────────────────────────────────────────────────────────────────────────────
def normalize_paths(text: str, bindir: Path) -> str:
    """Replace absolute paths with placeholders for reproducible diffing."""
    suite_root = str(JUDGER_DIR / "yaml-test-suite")
    text = text.replace(str(bindir) + "/", "<BIN>/")
    text = text.replace(str(bindir), "<BIN>")
    text = text.replace(suite_root + "/", "<INPUT>/")
    text = text.replace(suite_root, "<INPUT>")
    text = text.replace(str(JUDGER_DIR), "<JUDGER>")
    return text


# ─────────────────────────────────────────────────────────────────────────────
# Running one test case (isolated subprocess)
# ─────────────────────────────────────────────────────────────────────────────
def run_one_case(case: TestCase, binary: Path, timeout_sec: int, bindir: Path) -> TestResult:
    """Run one test case as a subprocess. Captures stdout, stderr, exit code.

    Subprocess isolation: a crash or hang in this case cannot affect others.
      - timeout=N kills the process after N seconds (sets returncode to 124)
      - SIGSEGV/SIGABRT/etc. set returncode to 128 + signal_number
    """
    if case.input_mode == "arg" or case.input_mode == "event":
        cmd = [str(binary), case.input_path]
        stdin_data = None
    elif case.input_mode == "stdin":
        cmd = [str(binary)]
        stdin_data = Path(case.input_path).read_bytes()
    elif case.input_mode == "self":
        cmd = [str(binary)]
        stdin_data = None
    else:
        raise ValueError(f"unknown input_mode: {case.input_mode}")

    timed_out = False
    signal_num = None
    try:
        result = subprocess.run(
            cmd,
            input=stdin_data,
            capture_output=True,
            timeout=timeout_sec,
        )
        exit_code = result.returncode
        stdout = result.stdout
        stderr = result.stderr
        if exit_code < 0:
            # subprocess.run returns negative if killed by signal
            signal_num = -exit_code
            exit_code = 128 + signal_num
    except subprocess.TimeoutExpired as e:
        timed_out = True
        exit_code = 124
        stdout = e.stdout or b""
        stderr = e.stderr or b""

    return TestResult(
        function=case.function,
        input=case.input,
        exit_code=exit_code,
        timeout=timed_out,
        signal=signal_num,
        stdout=normalize_paths(stdout.decode("utf-8", errors="replace"), bindir),
        stderr=normalize_paths(stderr.decode("utf-8", errors="replace"), bindir),
    )


# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────
def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--lib", required=True, help="Path to libyaml.a")
    p.add_argument("--outdir", required=True, help="Output directory")
    p.add_argument("--timeout", type=int, default=10, help="Per-case timeout in seconds")
    p.add_argument("--include", default=str(JUDGER_DIR.parent / "include"))
    p.add_argument("--src", default=str(JUDGER_DIR.parent / "src"))
    p.add_argument("--cc", default="cc")
    args = p.parse_args()

    lib_path = Path(args.lib).resolve()
    outdir = Path(args.outdir).resolve()
    include_dir = Path(args.include).resolve()
    src_dir = Path(args.src).resolve()

    if not lib_path.is_file():
        print(f"Error: {lib_path} does not exist", file=sys.stderr)
        return 1

    outdir.mkdir(parents=True, exist_ok=True)
    bindir = outdir / "bin"

    print(f"[run] Library:  {lib_path}")
    print(f"[run] Output:   {outdir}")
    print(f"[run] Compiling test functions...")
    binaries = compile_test_functions(bindir, lib_path, include_dir, src_dir, cc=args.cc)
    print(f"[run] Compiled {len(binaries)}/{len(ALL_FUNCS)} test functions")

    cases = enumerate_test_cases()
    print(f"[run] Running {len(cases)} test cases (timeout={args.timeout}s)...")

    results_path = outdir / "results.jsonl"
    n_done = 0
    with results_path.open("w") as f:
        for case in cases:
            if case.function not in binaries:
                continue
            result = run_one_case(case, binaries[case.function], args.timeout, bindir)
            f.write(json.dumps(asdict(result), sort_keys=True) + "\n")
            n_done += 1
            if n_done % 500 == 0:
                print(f"[run]   {n_done}/{len(cases)} done")

    print(f"[run] Done. {n_done} results → {results_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
