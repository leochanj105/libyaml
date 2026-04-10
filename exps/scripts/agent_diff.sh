#!/bin/bash
set -e

# agent_diff.sh — compile and run difftest_suite.c under one config,
# linking both the C library and the Rust static library.
#
# Usage: agent_diff.sh [-o OUTDIR] <config>
#
# Uses get_compile_args(), get_cargo_features(), and get_c_sources_for_config()
# from config.sh (overridden by project config_overrides.sh).

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../config.sh"

usage() {
    echo "Usage: $0 [-o OUTDIR] <config>"
    exit 1
}

OUTDIR="${PWD}/diffgen"
while getopts "o:" opt; do
    case $opt in
        o) OUTDIR="$OPTARG" ;;
        *) usage ;;
    esac
done
shift $((OPTIND - 1))

case "$OUTDIR" in /*) ;; *) OUTDIR="${PWD}/${OUTDIR}" ;; esac

[ $# -eq 1 ] || usage

CONFIG="$1"
TEST_FILE="${OUTDIR}/difftest_suite.c"
OUTPUT_DIR="${OUTDIR}/difffeedback"

[ -f "$TEST_FILE" ] || { echo "Error: difftest_suite.c not found: $TEST_FILE"; exit 1; }

# Get compile args and cargo features from config (project-specific overrides)
COMPILE_ARGS=$(get_compile_args "$CONFIG")
CARGO_FEATURES=$(get_cargo_features "$CONFIG")

# Get C source files — use project override if available, else generic
if type get_c_sources_for_config &>/dev/null; then
    C_SRCS=$(get_c_sources_for_config "$CONFIG")
else
    C_SRCS=$(get_c_sources)
fi

# Extract ALL test IDs from difftest_suite.c
RUN_MACROS=""
for test_id in $(grep -oP '(?<=#ifdef RUN_)[A-Za-z_0-9]+' "$TEST_FILE" | sort -u); do
    RUN_MACROS="$RUN_MACROS -DRUN_${test_id}"
done

# ---------------------------------------------------------------------------
# Step 1: Build the Rust static library
# ---------------------------------------------------------------------------
# Group configs that share the same cargo features into a single target dir.
# This avoids rebuilding identical Rust code for every config variant.
_feat_key=$(echo "${CARGO_FEATURES:-default}" | tr ' ,' '__')
RUST_TARGET_DIR="${OUTDIR}/rust_build/feat_${_feat_key}"
mkdir -p "${RUST_TARGET_DIR}"

CARGO_ARGS="--release --lib --manifest-path ${RUST_DIR}/Cargo.toml --target-dir ${RUST_TARGET_DIR}"
if [ -n "$CARGO_FEATURES" ]; then
    CARGO_ARGS="$CARGO_ARGS --no-default-features --features ${CARGO_FEATURES}"
fi

echo "=== Building Rust static library (config: ${CONFIG}, features: ${CARGO_FEATURES:-default}) ==="
cargo build $CARGO_ARGS

# Find the Rust static library
RUST_LIB=$(find "${RUST_TARGET_DIR}/release" -name '*.a' -type f | head -1)
[ -n "$RUST_LIB" ] || { echo "Error: no .a found in ${RUST_TARGET_DIR}/release"; exit 1; }

# ---------------------------------------------------------------------------
# Step 2: Build the C library in an isolated temp dir
# ---------------------------------------------------------------------------
BUILDDIR=$(mktemp -d)
trap 'rm -rf "$BUILDDIR"' EXIT
cd "$BUILDDIR"

echo ""
echo "=== Building C library ==="
$CC $COMPILE_ARGS -c $C_SRCS
ar rcs libmylib.a ./*.o
rm -f ./*.o

# If test_bridge.c exists, compile and add to library (exposes static functions)
BRIDGE_FILE="${OUTDIR}/test_bridge.c"
[ -f "$BRIDGE_FILE" ] || BRIDGE_FILE="${OUTDIR}/../testgen/test_bridge.c"
if [ -f "$BRIDGE_FILE" ]; then
    echo "  Including test_bridge.c (static function wrappers)"
    $CC $COMPILE_ARGS -c "$BRIDGE_FILE" -o bridge.o 2>/dev/null &&         ar rcs libmylib.a bridge.o && rm -f bridge.o ||         echo "  WARNING: test_bridge.c failed to compile"
fi

# ---------------------------------------------------------------------------
# Step 3: Compile and run the differential test
# ---------------------------------------------------------------------------
echo ""
echo "=== Compiling differential test binary (config: ${CONFIG}) ==="
$CC $COMPILE_ARGS \
    ${RUN_MACROS} \
    "${TEST_FILE}" \
    ./libmylib.a "${RUST_LIB}" \
    -lm -lpthread -ldl -lrt -lutil -lssl -lcrypto -lgcc_s \
    -o difftest_bin 2>"${BUILDDIR}/compile_err.txt" || {
        mkdir -p "${OUTPUT_DIR}"
        ERROR_FILE="${OUTPUT_DIR}/${CONFIG}_diff_error"
        echo "COMPILE ERROR for config ${CONFIG}" > "$ERROR_FILE"
        cat "${BUILDDIR}/compile_err.txt" >> "$ERROR_FILE"
        echo "Compilation failed for ${CONFIG}"
        cat "$ERROR_FILE"
        exit 1
    }

echo ""
echo "=== Running differential tests (config: ${CONFIG}) ==="
mkdir -p "${OUTPUT_DIR}"
RESULT_FILE="${OUTPUT_DIR}/${CONFIG}_diffresult"
STDERR_FILE="${BUILDDIR}/difftest_stderr.txt"

export RUST_BACKTRACE=1

set +e
./difftest_bin > "$RESULT_FILE" 2>"$STDERR_FILE"
EXIT_CODE=$?
set -e

if [ -s "$STDERR_FILE" ]; then
    echo "" >> "$RESULT_FILE"
    echo "=== STDERR OUTPUT ===" >> "$RESULT_FILE"
    cat "$STDERR_FILE" >> "$RESULT_FILE"
fi

if [ "$EXIT_CODE" -ne 0 ]; then
    echo "" >> "$RESULT_FILE"
    if [ "$EXIT_CODE" -eq 139 ]; then
        echo "RUNTIME ERROR: Segmentation fault (exit code 139)" >> "$RESULT_FILE"
    elif [ "$EXIT_CODE" -eq 134 ]; then
        echo "RUNTIME ERROR: Abort/panic (exit code 134)" >> "$RESULT_FILE"
    else
        echo "RUNTIME ERROR: Process exited with code ${EXIT_CODE}" >> "$RESULT_FILE"
    fi
fi

cat "$RESULT_FILE"

if [ "$EXIT_CODE" -ne 0 ]; then
    echo "Differential test failed with exit code ${EXIT_CODE}"
    exit "$EXIT_CODE"
fi
