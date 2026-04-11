#!/usr/bin/env bash
# config_overrides.sh — S1: naive one-shot test generation (no coverage feedback)

LIBYAML_DIR="/home/leochanj/Desktop/libyaml"

: "${TEST_CASE_DIR:=${LIBYAML_DIR}}"
: "${RUST_DIR:=${LIBYAML_DIR}/rust-s1}"
: "${WORK_DIR:=${LIBYAML_DIR}/exps/work-s1}"

: "${C_SRC_DIRS:=${TEST_CASE_DIR}/src}"
C_INCLUDE_DIRS="${TEST_CASE_DIR}/include ${TEST_CASE_DIR}/build/include"

# S1: no coverage feedback, single round
: "${COVERAGE_MODES:=none}"
: "${MAX_ROUNDS:=1}"

# Judger
: "${JUDGER_DIR:=${LIBYAML_DIR}/testing}"
: "${JUDGER_SCRIPT:=${LIBYAML_DIR}/exps/judger.sh}"

# Compile args (need HAVE_CONFIG_H for yaml_private.h)
get_compile_args() {
    echo "-DHAVE_CONFIG_H=1 -I${TEST_CASE_DIR}/include -I${TEST_CASE_DIR}/build/include -I${TEST_CASE_DIR}/src"
}

LLVM_PROFDATA=llvm-profdata-21
LLVM_COV=llvm-cov-21

: "${CODE_GEN_CMD:=claude}"
: "${ANALYSIS_CMD:=claude}"
