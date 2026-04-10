#!/usr/bin/env bash
set -euo pipefail

# build_and_cover.sh — compile test_suite.c with LLVM coverage, run, emit feedback.
# Standalone version (no harness config.sh dependency).
#
# Usage: build_and_cover.sh -w WORKDIR <config>
#
# Reads env vars: CC, LLVM_PROFDATA, LLVM_COV, C_SRC_DIRS, C_INCLUDE_DIRS

CC="${CC:-clang-21}"
LLVM_PROFDATA="${LLVM_PROFDATA:-llvm-profdata-21}"
LLVM_COV="${LLVM_COV:-llvm-cov-21}"

WORKDIR=""
while getopts "w:" opt; do
    case $opt in w) WORKDIR="$OPTARG" ;; *) echo "Usage: $0 -w WORKDIR <config>"; exit 1 ;; esac
done
shift $((OPTIND - 1))

[ -z "$WORKDIR" ] && { echo "Usage: $0 -w WORKDIR <config>"; exit 1; }
[ $# -eq 1 ] || { echo "Usage: $0 -w WORKDIR <config>"; exit 1; }

case "$WORKDIR" in /*) ;; *) WORKDIR="${PWD}/${WORKDIR}" ;; esac

CONFIG="$1"
TEST_SUITE="${WORKDIR}/test_suite.c"

[ -f "$TEST_SUITE" ] || { echo "Error: test_suite.c not found in ${WORKDIR}"; exit 1; }

# Compile args: just include dirs
INC_FLAGS=""
for d in $C_INCLUDE_DIRS; do INC_FLAGS="$INC_FLAGS -I$d"; done

# Collect C source files (exclude fenv.c)
C_SRCS=""
for d in $C_SRC_DIRS; do
    [ -d "$d" ] || continue
    while IFS= read -r f; do
        C_SRCS="$C_SRCS $f"
    done < <(find "$d" -name '*.c' -type f 2>/dev/null | grep -v fenv.c)
done

BUILDDIR=$(mktemp -d)
trap 'rm -rf "$BUILDDIR"' EXIT

PROFRAW="${BUILDDIR}/default.profraw"
PROFDATA="${BUILDDIR}/coverage.profdata"

echo "Building + covering: ${CONFIG}"
cd "$BUILDDIR"

# Compile C library with coverage
$CC $INC_FLAGS -fprofile-instr-generate -fcoverage-mapping -c $C_SRCS 2>/dev/null || {
    echo "  Library compile failed for ${CONFIG}" >&2
    mkdir -p "${WORKDIR}/feedback"
    echo "COMPILE_ERROR: library compilation failed" > "${WORKDIR}/feedback/${CONFIG}_feedback"
    exit 0
}
ar rcs libmylib.a ./*.o 2>/dev/null
rm -f ./*.o

# Compile test bridge if it exists
BRIDGE_FILE="${WORKDIR}/test_bridge.c"
if [ -f "$BRIDGE_FILE" ]; then
    _bridge_includes=""
    for _sd in $C_SRC_DIRS; do _bridge_includes="$_bridge_includes -I$_sd"; done
    echo "  Compiling test_bridge.c"
    if $CC $INC_FLAGS $_bridge_includes -fprofile-instr-generate -fcoverage-mapping \
        -c "$BRIDGE_FILE" -o bridge.o 2>"${BUILDDIR}/bridge_err.txt"; then
        ar rcs libmylib.a bridge.o
    else
        echo "  WARNING: test_bridge.c failed to compile"
    fi
fi

# Compile test suite WITHOUT coverage (we only want library coverage)
if ! $CC $INC_FLAGS -c "$TEST_SUITE" -o test_suite.o 2>"${BUILDDIR}/compile_err.txt"; then
    echo "  Test suite compile failed for ${CONFIG}"
    mkdir -p "${WORKDIR}/feedback"
    {
        echo "COMPILE_ERROR"
        cat "${BUILDDIR}/compile_err.txt"
    } > "${WORKDIR}/feedback/${CONFIG}_feedback"
    exit 0
fi

# Link with --whole-archive so all library functions get coverage
if ! $CC -fprofile-instr-generate \
    -Wl,--allow-multiple-definition \
    test_suite.o -Wl,--whole-archive ./libmylib.a -Wl,--no-whole-archive \
    -lm -o test_bin 2>"${BUILDDIR}/link_err.txt"; then
    echo "  Link failed for ${CONFIG}"
    mkdir -p "${WORKDIR}/feedback"
    {
        echo "COMPILE_ERROR: link failed"
        cat "${BUILDDIR}/link_err.txt"
    } > "${WORKDIR}/feedback/${CONFIG}_feedback"
    exit 0
fi

# Run and collect coverage
LLVM_PROFILE_FILE="$PROFRAW" ./test_bin >/dev/null 2>&1 || true

if [ ! -f "$PROFRAW" ]; then
    echo "  No coverage data for ${CONFIG}"
    mkdir -p "${WORKDIR}/feedback"
    echo "NO_COVERAGE: profraw not produced" > "${WORKDIR}/feedback/${CONFIG}_feedback"
    exit 0
fi

if ! $LLVM_PROFDATA merge -sparse "$PROFRAW" -o "$PROFDATA" 2>/dev/null; then
    echo "  profdata merge failed for ${CONFIG}"
    mkdir -p "${WORKDIR}/feedback"
    echo "NO_COVERAGE: profdata merge failed" > "${WORKDIR}/feedback/${CONFIG}_feedback"
    exit 0
fi

mkdir -p "${WORKDIR}/feedback"

# Export coverage as JSON (used by branch_coverage.py)
$LLVM_COV export ./test_bin \
    -instr-profile="$PROFDATA" \
    2>/dev/null > "${WORKDIR}/feedback/${CONFIG}_export.json"

# Also keep the show output for backward compatibility
$LLVM_COV show ./test_bin \
    -instr-profile="$PROFDATA" \
    --show-branches=count \
    2>/dev/null > "${WORKDIR}/feedback/${CONFIG}_feedback"

# Sanity: check export has data
if ! python3 -c "
import json, sys
d = json.load(open('${WORKDIR}/feedback/${CONFIG}_export.json'))
branches = sum(len(f.get('branches',[])) for f in d['data'][0]['files'])
if branches == 0: sys.exit(1)
" 2>/dev/null; then
    echo "  WARNING: no branch data in export for ${CONFIG}"
fi

echo "  Done: ${CONFIG}"
