#!/usr/bin/env bash
# run-tests.sh — Run all judger test suites, collect outputs for bitwise comparison.
#
# Each test case = one test function × one test input, run as an isolated subprocess.
# A crash or hang in one test case cannot affect others.
#
# Output: one deterministic file per suite (suite1.out, suite2.out)
# suitable for `diff` or `cmp` between two library versions.
#
# Usage:
#   ./run-tests.sh --outdir=<dir> [--suite=1|2] [--timeout=SEC]
#
# Required environment:
#   LIBYAML_LIB   - path to libyaml .a to link against
#
# Optional environment:
#   CC            - compiler (default: cc)
#   CFLAGS        - extra compiler flags
#   INCLUDE_DIR   - path to libyaml include/ (default: ../include)
#   SRC_DIR       - path to libyaml src/ (default: ../src, needed for private headers)
#   TIMEOUT_SEC   - per-test-case timeout in seconds (default: 10)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CC="${CC:-cc}"
CFLAGS="${CFLAGS:-}"
LIBYAML_LIB="${LIBYAML_LIB:-}"
INCLUDE_DIR="${INCLUDE_DIR:-${SCRIPT_DIR}/../include}"
SRC_DIR="${SRC_DIR:-${SCRIPT_DIR}/../src}"
OUTDIR=""
SUITE_FILTER=""
TIMEOUT_SEC="${TIMEOUT_SEC:-10}"

for arg in "$@"; do
    case "$arg" in
        --outdir=*) OUTDIR="${arg#--outdir=}" ;;
        --suite=*) SUITE_FILTER="${arg#--suite=}" ;;
        --timeout=*) TIMEOUT_SEC="${arg#--timeout=}" ;;
        -h|--help)
            echo "Usage: $0 --outdir=<dir> [--suite=1|2] [--timeout=SEC]"
            echo "  Suite 1: yaml-test-suite × test functions"
            echo "  Suite 2: self-contained tests (test-version, test-reader)"
            exit 0 ;;
    esac
done

if [ -z "$OUTDIR" ]; then
    echo "Error: --outdir=<dir> required" >&2; exit 1
fi
if [ -z "$LIBYAML_LIB" ]; then
    echo "Error: LIBYAML_LIB must be set" >&2; exit 1
fi

mkdir -p "$OUTDIR"

BINDIR=$(mktemp -d /tmp/judger-bin.XXXXXX)
trap "rm -rf $BINDIR" EXIT

# =========================================================================
# Compile all test functions
# =========================================================================
log() { echo "[judger] $*" >&2; }

compile() {
    local src="$1" bin="$2" extra_flags="${3:-}"
    $CC $CFLAGS $extra_flags -I"$INCLUDE_DIR" -o "$bin" "$src" "$LIBYAML_LIB" -lm 2>&1
}

log "Compiling test functions..."

# Test functions that take YAML file as argument
ARG_FUNCS=(run-scanner run-parser run-loader run-emitter run-dumper
           run-parser-test-suite)

# Test functions that read YAML from stdin
STDIN_FUNCS=(example-reformatter example-reformatter-alt
             example-deconstructor example-deconstructor-alt)

# Test function that takes event file as argument
EVENT_FUNCS=(run-emitter-test-suite)

# Self-contained test functions (no external input)
SELF_FUNCS=(test-version test-reader)

compile_failures=0
for name in "${ARG_FUNCS[@]}" "${STDIN_FUNCS[@]}" "${EVENT_FUNCS[@]}" "${SELF_FUNCS[@]}"; do
    src="${SCRIPT_DIR}/test-functions/${name}.c"
    extra=""
    # run-emitter-test-suite needs private headers
    if [ "$name" = "run-emitter-test-suite" ]; then
        extra="-I${SRC_DIR}/.."
    fi
    if ! compile "$src" "$BINDIR/$name" "$extra" > "$BINDIR/${name}.compile.log" 2>&1; then
        log "FAILED to compile $name (see $BINDIR/${name}.compile.log)"
        compile_failures=$((compile_failures + 1))
    fi
done

if [ "$compile_failures" -gt 0 ]; then
    log "WARNING: $compile_failures test function(s) failed to compile"
fi
log "Compilation done."

# =========================================================================
# Core: run one test case as isolated subprocess, emit deterministic record
# =========================================================================
run_case() {
    local label="$1" bin="$2"
    shift 2
    # remaining args are passed to the binary

    local tmp_stdout tmp_stderr rc
    tmp_stdout=$(mktemp)
    tmp_stderr=$(mktemp)

    # Run in isolated subprocess with timeout
    #   0-125: normal exit
    #   124: timeout (hung)
    #   128+N: killed by signal N (e.g. 139=SIGSEGV, 134=SIGABRT)
    # Suppress bash's "Aborted" job-control message on SIGABRT
    set +e
    { timeout "$TIMEOUT_SEC" "$bin" "$@" >"$tmp_stdout" 2>"$tmp_stderr"; } 2>/dev/null
    rc=$?
    set -e

    echo "=== CASE ${label} ==="
    if [ "$rc" -eq 124 ]; then
        echo "EXIT: ${rc} (TIMEOUT)"
    elif [ "$rc" -gt 128 ]; then
        echo "EXIT: ${rc} (SIGNAL $((rc - 128)))"
    else
        echo "EXIT: ${rc}"
    fi
    echo "STDOUT:"
    cat "$tmp_stdout"
    echo "STDERR:"
    cat "$tmp_stderr"
    echo "=== END ==="

    rm -f "$tmp_stdout" "$tmp_stderr"
}

# Run a test case with input piped via stdin
run_case_stdin() {
    local label="$1" bin="$2" input_file="$3"

    local tmp_stdout tmp_stderr rc
    tmp_stdout=$(mktemp)
    tmp_stderr=$(mktemp)

    set +e
    { timeout "$TIMEOUT_SEC" "$bin" <"$input_file" >"$tmp_stdout" 2>"$tmp_stderr"; } 2>/dev/null
    rc=$?
    set -e

    echo "=== CASE ${label} ==="
    if [ "$rc" -eq 124 ]; then
        echo "EXIT: ${rc} (TIMEOUT)"
    elif [ "$rc" -gt 128 ]; then
        echo "EXIT: ${rc} (SIGNAL $((rc - 128)))"
    else
        echo "EXIT: ${rc}"
    fi
    echo "STDOUT:"
    cat "$tmp_stdout"
    echo "STDERR:"
    cat "$tmp_stderr"
    echo "=== END ==="

    rm -f "$tmp_stdout" "$tmp_stderr"
}

# =========================================================================
# Suite 1: yaml-test-suite × test functions
# =========================================================================
if [ -z "$SUITE_FILTER" ] || [ "$SUITE_FILTER" = "1" ]; then
    log "Running suite 1 (yaml-test-suite)..."
    exec 3>"$OUTDIR/suite1.out"

    count=0
    # Find all in.yaml files, including sub-variants (e.g. 2G84/00/in.yaml)
    # Exclude name/ and tags/ metadata directories
    while IFS= read -r inyaml; do
        dir=$(dirname "$inyaml")
        # Build a label from the path relative to yaml-test-suite/
        label=$(realpath --relative-to="${SCRIPT_DIR}/yaml-test-suite" "$dir")

        # Arg-taking test functions × this input
        for func in "${ARG_FUNCS[@]}"; do
            bin="$BINDIR/$func"
            [ -x "$bin" ] || continue
            run_case "func=${func} input=${label}" "$bin" "$inyaml" >&3
            count=$((count + 1))
        done

        # Stdin-taking test functions × this input
        for func in "${STDIN_FUNCS[@]}"; do
            bin="$BINDIR/$func"
            [ -x "$bin" ] || continue
            run_case_stdin "func=${func} input=${label}" "$bin" "$inyaml" >&3
            count=$((count + 1))
        done

        # Event-taking test functions × this input's test.event
        if [ -e "$dir/test.event" ]; then
            for func in "${EVENT_FUNCS[@]}"; do
                bin="$BINDIR/$func"
                [ -x "$bin" ] || continue
                run_case "func=${func} input=${label}" "$bin" "$dir/test.event" >&3
                count=$((count + 1))
            done
        fi
    done < <(find "${SCRIPT_DIR}/yaml-test-suite" -name in.yaml \
                -not -path "*/name/*" -not -path "*/tags/*" | sort)

    exec 3>&-
    log "Suite 1: $count test cases → $OUTDIR/suite1.out"
fi

# =========================================================================
# Suite 2: self-contained test functions
# =========================================================================
if [ -z "$SUITE_FILTER" ] || [ "$SUITE_FILTER" = "2" ]; then
    log "Running suite 2 (self-contained tests)..."
    exec 3>"$OUTDIR/suite2.out"

    count=0
    for func in "${SELF_FUNCS[@]}"; do
        bin="$BINDIR/$func"
        [ -x "$bin" ] || continue
        run_case "func=${func}" "$bin" >&3
        count=$((count + 1))
    done

    exec 3>&-
    log "Suite 2: $count test cases → $OUTDIR/suite2.out"
fi

# =========================================================================
# Normalize: strip absolute paths for reproducible bitwise comparison
# =========================================================================
for f in "$OUTDIR"/suite*.out; do
    [ -f "$f" ] || continue
    sed -i \
        -e "s|${BINDIR}/[^ ']*|<BIN>|g" \
        -e "s|${SCRIPT_DIR}/yaml-test-suite/[^ ':]*[/]|<INPUT>/|g" \
        -e "s|${SCRIPT_DIR}/[^ ']*|<JUDGER>|g" \
        "$f"
done

log "Done. Results in $OUTDIR/"
