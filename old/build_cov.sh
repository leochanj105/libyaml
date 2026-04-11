#!/usr/bin/env bash
set -euo pipefail

# build_cov.sh — Build libyaml as a static library with LLVM coverage instrumentation.
#
# Usage:
#   ./build_cov.sh                         # build library only
#   ./build_cov.sh test_suite.c            # build library + link test binary
#   ./build_cov.sh test_suite.c --run      # build + run + show coverage report
#
# Output (in ./build_cov/):
#   libyaml_cov.a     — static library with coverage instrumentation
#   test_bin           — test binary (if test_suite.c provided)
#   coverage.profdata  — merged profile (after --run)
#   export.json        — llvm-cov export JSON (after --run)
#
# Environment:
#   CC              — compiler (default: clang-21)
#   LLVM_PROFDATA   — profdata tool (default: llvm-profdata-21)
#   LLVM_COV        — cov tool (default: llvm-cov-21)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIBYAML_DIR="$(dirname "$SCRIPT_DIR")"

: "${CC:=clang-21}"
: "${LLVM_PROFDATA:=llvm-profdata-21}"
: "${LLVM_COV:=llvm-cov-21}"

SRC_DIR="${LIBYAML_DIR}/src"
INC_FLAGS="-DHAVE_CONFIG_H=1 -I${LIBYAML_DIR}/include -I${LIBYAML_DIR}/build/include -I${SRC_DIR}"

BUILD_DIR="${SCRIPT_DIR}/build_cov"
mkdir -p "$BUILD_DIR"

TEST_FILE=""
DO_RUN=0
WITH_BRIDGES=0
for arg in "$@"; do
    case "$arg" in
        --run)           DO_RUN=1 ;;
        --with-bridges)  WITH_BRIDGES=1 ;;
        *)               TEST_FILE="$arg" ;;
    esac
done

BRIDGE_DIR="${BUILD_DIR}/bridges"

# ── Step 1: Compile library sources with coverage ──
# When --with-bridges is set, we compile bridge_<src>.c instead of <src>.c for
# each source that has a bridge file. The bridge file #include's the source,
# so the source's code still gets instrumented, but it's attributed to the
# bridge object only — no duplication.
echo "=== Compiling libyaml with coverage ==="
OBJS=()
for src in "${SRC_DIR}"/*.c; do
    base=$(basename "$src" .c)
    bridge_src="${BRIDGE_DIR}/bridge_${base}.c"
    if [ "$WITH_BRIDGES" -eq 1 ] && [ -f "$bridge_src" ]; then
        obj="${BUILD_DIR}/bridge_${base}.o"
        $CC $INC_FLAGS -fprofile-instr-generate -fcoverage-mapping -O0 \
            -c "$bridge_src" -o "$obj"
        OBJS+=("$obj")
    else
        obj="${BUILD_DIR}/${base}.o"
        $CC $INC_FLAGS -fprofile-instr-generate -fcoverage-mapping -O0 \
            -c "$src" -o "$obj"
        OBJS+=("$obj")
    fi
done
echo "  Compiled ${#OBJS[@]} object files"

# ── Step 2: Create static archive ──
ar rcs "${BUILD_DIR}/libyaml_cov.a" "${OBJS[@]}"
echo "  Created: ${BUILD_DIR}/libyaml_cov.a"

# Clean up object files
rm -f "${OBJS[@]}"

# ── Step 3: Link test binary (if test file provided) ──
if [ -n "$TEST_FILE" ]; then
    [ -f "$TEST_FILE" ] || { echo "Error: $TEST_FILE not found"; exit 1; }
    echo ""
    echo "=== Compiling test binary ==="

    # Compile test file WITHOUT coverage (we only want library coverage)
    $CC $INC_FLAGS -O0 -c "$TEST_FILE" -o "${BUILD_DIR}/test_suite.o" 2>&1
    echo "  Compiled: $TEST_FILE"

    # Link: --whole-archive ensures all library functions get coverage even if unreferenced
    $CC -fprofile-instr-generate \
        "${BUILD_DIR}/test_suite.o" \
        -Wl,--whole-archive "${BUILD_DIR}/libyaml_cov.a" -Wl,--no-whole-archive \
        -Wl,--allow-multiple-definition \
        -lm -o "${BUILD_DIR}/test_bin" 2>&1
    echo "  Linked:   ${BUILD_DIR}/test_bin"

    rm -f "${BUILD_DIR}/test_suite.o"
fi

# ── Step 4: Run + coverage report (if --run) ──
if [ "$DO_RUN" -eq 1 ] && [ -f "${BUILD_DIR}/test_bin" ]; then
    echo ""
    echo "=== Running test binary ==="
    LLVM_PROFILE_FILE="${BUILD_DIR}/default.profraw" "${BUILD_DIR}/test_bin" || true

    if [ ! -f "${BUILD_DIR}/default.profraw" ]; then
        echo "  ERROR: no profraw produced"
        exit 1
    fi

    $LLVM_PROFDATA merge -sparse "${BUILD_DIR}/default.profraw" -o "${BUILD_DIR}/coverage.profdata"
    echo "  Merged profile: ${BUILD_DIR}/coverage.profdata"

    # Export JSON (for programmatic use)
    $LLVM_COV export "${BUILD_DIR}/test_bin" \
        -instr-profile="${BUILD_DIR}/coverage.profdata" \
        > "${BUILD_DIR}/export.json" 2>/dev/null

    echo ""
    echo "=== Coverage Report ==="
    $LLVM_COV report "${BUILD_DIR}/test_bin" \
        -instr-profile="${BUILD_DIR}/coverage.profdata" \
        -sources "${SRC_DIR}"

    echo ""
    echo "=== Summary ==="
    echo "  Profile:    ${BUILD_DIR}/coverage.profdata"
    echo "  Export:     ${BUILD_DIR}/export.json"
    echo ""
    echo "For detailed per-file view:"
    echo "  $LLVM_COV show ${BUILD_DIR}/test_bin -instr-profile=${BUILD_DIR}/coverage.profdata -sources ${SRC_DIR} --show-branches=count"
    echo ""
    echo "For uncovered functions:"
    echo "  $LLVM_COV report ${BUILD_DIR}/test_bin -instr-profile=${BUILD_DIR}/coverage.profdata -sources ${SRC_DIR} -show-functions | grep '0.00%'"
fi

echo ""
echo "Done."
