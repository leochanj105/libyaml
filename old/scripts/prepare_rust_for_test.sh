#!/usr/bin/env bash
set -euo pipefail

# prepare_rust_for_test.sh — prepare transpiled Rust for differential testing.
#
# Takes rust-baseline (pure transpilation output) and produces
# rust-baseline-test (with test bridge for difftest linking).
#
# Changes made:
#   1. Copies rust-baseline → rust-baseline-test
#   2. Makes internal functions pub(crate) so test_bridge.rs can call them
#   3. Copies test_bridge.rs into src/
#   4. Adds "mod test_bridge;" to lib.rs
#   5. Verifies cargo build succeeds
#
# Usage: prepare_rust_for_test.sh [EXP_DIR]
#   EXP_DIR defaults to the parent directory of this script's location.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXP_DIR="${1:-$(dirname "$SCRIPT_DIR")}"

BASELINE="${EXP_DIR}/rust-baseline"
OUTPUT="${EXP_DIR}/rust-baseline-test"
BRIDGE_SRC="${EXP_DIR}/test_bridge.rs"

[ -d "$BASELINE" ] || { echo "ERROR: rust-baseline not found at $BASELINE" >&2; exit 1; }
[ -f "$BRIDGE_SRC" ] || { echo "ERROR: test_bridge.rs not found at $BRIDGE_SRC" >&2; exit 1; }

echo "Preparing rust-baseline-test..."
echo "  Source:  $BASELINE"
echo "  Output:  $OUTPUT"
echo "  Bridge:  $BRIDGE_SRC"

# ── Step 1: Copy baseline ──
rm -rf "$OUTPUT"
cp -r "$BASELINE" "$OUTPUT"
# Remove cached build artifacts so we get a clean build
rm -rf "$OUTPUT/target"

SRC="${OUTPUT}/src"

# ── Step 2: Make internal functions pub(crate) ──
# These functions exist in the transpiled code as private (fn) or public (pub fn).
# test_bridge.rs needs pub(crate) access to call them.
# We use sed to change "fn name(" → "pub(crate) fn name(" for private ones.
# Already-public ones (pub fn) are left as-is — they're already accessible.

echo "  Adjusting function visibility..."

# mathd.rs: tan_kern, sin_pi (private)
# cos_kern, sin_kern, rem_pio2_internal, rem_pio2, lgamma_r are already pub
sed -i 's/^fn tan_kern(/pub(crate) fn tan_kern(/' "$SRC/mathd.rs"
sed -i 's/^fn sin_pi(/pub(crate) fn sin_pi(/' "$SRC/mathd.rs"

# mathf.rs: all private
sed -i 's/^fn tanf_kern(/pub(crate) fn tanf_kern(/' "$SRC/mathf.rs"
sed -i 's/^fn sin_pif(/pub(crate) fn sin_pif(/' "$SRC/mathf.rs"
sed -i 's/^fn cosf_kern(/pub(crate) fn cosf_kern(/' "$SRC/mathf.rs"
sed -i 's/^fn sinf_kern(/pub(crate) fn sinf_kern(/' "$SRC/mathf.rs"
sed -i 's/^fn rem_pio2f_internal(/pub(crate) fn rem_pio2f_internal(/' "$SRC/mathf.rs"
sed -i 's/^fn rem_pio2f_fn(/pub(crate) fn rem_pio2f_fn(/' "$SRC/mathf.rs"
sed -i 's/^fn lgammaf_r(/pub(crate) fn lgammaf_r(/' "$SRC/mathf.rs"

# complexd.rs: all private
sed -i 's/^fn ctans(/pub(crate) fn ctans(/' "$SRC/complexd.rs"
sed -i 's/^fn redupi(/pub(crate) fn redupi(/' "$SRC/complexd.rs"
sed -i 's/^fn ccoshsinh(/pub(crate) fn ccoshsinh(/' "$SRC/complexd.rs"

# complexf.rs: all private
sed -i 's/^fn ctansf(/pub(crate) fn ctansf(/' "$SRC/complexf.rs"
sed -i 's/^fn redupif(/pub(crate) fn redupif(/' "$SRC/complexf.rs"
sed -i 's/^fn ccoshsinhf(/pub(crate) fn ccoshsinhf(/' "$SRC/complexf.rs"

# ── Step 3: Copy test_bridge.rs ──
cp "$BRIDGE_SRC" "$SRC/test_bridge.rs"

# ── Step 4: Add mod test_bridge to lib.rs ──
if ! grep -q 'mod test_bridge' "$SRC/lib.rs"; then
    sed -i '/^pub mod complexf;/a mod test_bridge;' "$SRC/lib.rs"
fi

# ── Step 5: Verify build ──
echo "  Building..."
if cargo build --release --lib --manifest-path "$OUTPUT/Cargo.toml" 2>"${OUTPUT}/build_err.txt"; then
    echo "  Build OK"
    rm -f "${OUTPUT}/build_err.txt"
else
    echo "  BUILD FAILED:" >&2
    cat "${OUTPUT}/build_err.txt" >&2
    exit 1
fi

echo "Done: $OUTPUT"
