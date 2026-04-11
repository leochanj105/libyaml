#!/usr/bin/env bash
# config_overrides.sh — S4: function-coverage feedback loop (multi-round)

LIBYAML_DIR="/home/leochanj/Desktop/libyaml"

: "${TEST_CASE_DIR:=${LIBYAML_DIR}}"
: "${RUST_DIR:=${LIBYAML_DIR}/rust-s4}"
: "${WORK_DIR:=${LIBYAML_DIR}/exps/work/s4_function}"

: "${C_SRC_DIRS:=${TEST_CASE_DIR}/src}"
C_INCLUDE_DIRS="${TEST_CASE_DIR}/include ${TEST_CASE_DIR}/build/include"

# S4: function coverage feedback loop
: "${COVERAGE_MODES:=function}"
: "${MAX_ROUNDS:=8}"
: "${STALL_LIMIT:=3}"

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
