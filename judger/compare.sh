#!/usr/bin/env bash
# compare.sh — Compare outputs from two judger runs.
#
# Usage: ./compare.sh <dir-A> <dir-B>

set -euo pipefail

DIR_A="$1"
DIR_B="$2"

rc=0
for suite in suite1.out suite2.out; do
    echo "=== $suite ==="
    if diff "$DIR_A/$suite" "$DIR_B/$suite"; then
        echo "IDENTICAL"
    else
        rc=1
    fi
    echo ""
done

exit $rc
