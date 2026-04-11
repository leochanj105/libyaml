#!/usr/bin/env bash
# config_overrides.sh — S2: explicit function-coverage prompt (no feedback loop)

LIBYAML_DIR="/home/leochanj/Desktop/libyaml"

: "${TEST_CASE_DIR:=${LIBYAML_DIR}}"
: "${RUST_DIR:=${LIBYAML_DIR}/rust-s2}"
: "${WORK_DIR:=${LIBYAML_DIR}/exps/work/s2_explicit}"

: "${C_SRC_DIRS:=${TEST_CASE_DIR}/src}"
C_INCLUDE_DIRS="${TEST_CASE_DIR}/include ${TEST_CASE_DIR}/build/include"

# S2: no coverage feedback, single round, but prompt is more detailed
: "${COVERAGE_MODES:=none}"
: "${MAX_ROUNDS:=1}"

# Judger
: "${JUDGER_DIR:=${LIBYAML_DIR}/testing}"
: "${JUDGER_SCRIPT:=${LIBYAML_DIR}/exps/judger.sh}"

get_compile_args() {
    echo "-DHAVE_CONFIG_H=1 -I${TEST_CASE_DIR}/include -I${TEST_CASE_DIR}/build/include -I${TEST_CASE_DIR}/src"
}

LLVM_PROFDATA=llvm-profdata-21
LLVM_COV=llvm-cov-21

: "${CODE_GEN_CMD:=claude}"
: "${ANALYSIS_CMD:=claude}"
