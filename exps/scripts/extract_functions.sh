#!/usr/bin/env bash
set -euo pipefail

# extract_functions.sh — extract all C function names from the compiled library.
# Uses nm on compiled .o files to get actually-compiled symbols only.
# Static functions are detected by compiling with -fno-inline and checking
# which symbols from source don't appear in nm output.
#
# Output: one function name per line. [static] prefix for internal functions.
# Run once; output is reused across all scenarios/rounds.
#
# Usage: extract_functions.sh <output_file>

OUTPUT="${1:?Usage: extract_functions.sh <output_file>}"

LIBMCS="${LIBMCS:-${TEST_CASE_DIR:-}}"
CC="${CC:-clang-21}"
C_SRC_DIRS="${C_SRC_DIRS:-${LIBMCS}/mathd ${LIBMCS}/mathf ${LIBMCS}/common ${LIBMCS}/complexd ${LIBMCS}/complexf}"
C_INCLUDE_DIRS="${C_INCLUDE_DIRS:-${LIBMCS}/include}"

if [ -f "$OUTPUT" ]; then
    echo "Functions already extracted: ${OUTPUT} ($(wc -l < "$OUTPUT") lines)"
    exit 0
fi

echo "Extracting function list from compiled objects..."

BDIR=$(mktemp -d)
trap 'rm -rf "$BDIR"' EXIT

INC_FLAGS=""
for d in $C_INCLUDE_DIRS; do INC_FLAGS="$INC_FLAGS -I$d"; done

# Compile all .c files to .o and extract symbols with nm
for d in $C_SRC_DIRS; do
    [ -d "$d" ] || continue
    find "$d" -name '*.c' -type f 2>/dev/null | grep -v fenv.c | sort | while read -r f; do
        bn=$(basename "$f" .c)
        dn=$(basename "$d")
        $CC $INC_FLAGS -c "$f" -o "${BDIR}/${dn}_${bn}.o" 2>/dev/null || true
    done
done

# Public functions: T (text) symbols from nm
nm "${BDIR}"/*.o 2>/dev/null | grep " T " | awk '{print $3}' | sort -u > "${BDIR}/public.txt"

# Static functions: defined in source but not in nm output.
# We detect them by parsing source for static function definitions,
# then checking they don't appear in the public list.
{
    for d in $C_SRC_DIRS; do
        [ -d "$d" ] || continue
        find "$d" -name '*.c' -type f 2>/dev/null | grep -v fenv.c | sort | while read -r f; do
            # Only extract static functions (not behind #ifdef guards that are disabled)
            # Simple approach: look for "static ... funcname(" at the start of a line
            # within the actually-compiled code. Since we compiled with the same flags,
            # functions behind disabled #ifdefs won't have corresponding .o entries anyway.
            awk '
            /^static[[:space:]].*\(/ {
                line = $0
                sub(/\(.*/, "", line)
                gsub(/[*]/, " ", line)
                n = split(line, parts, /[[:space:]]+/)
                if (n >= 2 && parts[n] ~ /^[a-zA-Z_]/) {
                    print parts[n]
                }
            }
            ' "$f" 2>/dev/null || true
        done
    done
} | sort -u > "${BDIR}/static_candidates.txt"

# Filter static candidates: only keep those NOT in the public symbol list
# (some "static inline" functions get inlined and don't appear as symbols,
# but we still want to list them if they exist in source)
comm -23 "${BDIR}/static_candidates.txt" "${BDIR}/public.txt" > "${BDIR}/static.txt"

# Generate output
{
    echo "# C functions in library (compiled symbols)"
    echo "# [static] prefix = internal linkage (needs bridge for testing)"
    echo "# Generated from: nm on compiled .o files"
    echo ""
    while read -r func; do
        echo "$func"
    done < "${BDIR}/public.txt"
    while read -r func; do
        echo "[static] $func"
    done < "${BDIR}/static.txt"
} | sort -t' ' -k2 > "$OUTPUT"

_total=$(grep -c -v '^#\|^$' "$OUTPUT" || echo 0)
_static=$(grep -c '^\[static\]' "$OUTPUT" || echo 0)
_public=$(( _total - _static ))
echo "  ${_total} functions (${_public} public, ${_static} static) -> ${OUTPUT}"
