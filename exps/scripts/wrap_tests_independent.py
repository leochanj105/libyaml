#!/usr/bin/env python3
"""wrap_tests_independent.py — post-process test_suite.c to run each test in a fork.

Reads a test suite C file, finds test function calls in main(), and wraps each
in a fork+waitpid so crashes/hangs in one test don't affect others.

Usage: python3 wrap_tests_independent.py input.c output.c [timeout_sec]
"""

import re
import sys

WRAPPER_CODE = r"""
/* ── Independent test execution wrapper ── */
#include <unistd.h>
#include <sys/wait.h>
#include <signal.h>
#include <string.h>

static void run_test(const char *name, void (*fn)(void), int timeout_sec) {
    fflush(stdout);
    fflush(stderr);
    pid_t pid = fork();
    if (pid == 0) {
        /* Child: run the test, exit */
        fn();
        fflush(stdout);
        _exit(0);
    }
    /* Parent: wait with timeout using alarm */
    int status;
    /* First try non-blocking wait — most tests finish instantly */
    usleep(1000); /* 1ms grace period */
    pid_t r = waitpid(pid, &status, WNOHANG);
    if (r == pid) goto done;
    if (r < 0) goto done;
    /* Still running — poll with 100ms intervals up to timeout */
    int polls = timeout_sec * 10; /* 100ms per poll */
    for (int i = 0; i < polls; i++) {
        usleep(100000); /* 100ms */
        r = waitpid(pid, &status, WNOHANG);
        if (r == pid) goto done;
        if (r < 0) goto done;
    }
    /* Timeout — kill child */
    kill(pid, SIGKILL);
    waitpid(pid, &status, 0);
    printf("FAULT %s TIMEOUT\n", name);
    fflush(stdout);
    return;
done:
    if (WIFSIGNALED(status)) {
        printf("FAULT %s SIGNAL %d\n", name, WTERMSIG(status));
        fflush(stdout);
    } else if (WIFEXITED(status) && WEXITSTATUS(status) != 0) {
        printf("FAULT %s EXIT %d\n", name, WEXITSTATUS(status));
        fflush(stdout);
    }
}
/* ── End wrapper ── */
"""

def transform(input_path, output_path, timeout_sec=5):
    with open(input_path) as f:
        source = f.read()

    # Find main() function body
    main_match = re.search(r'(int\s+main\s*\([^)]*\)\s*\{)', source)
    if not main_match:
        print("ERROR: could not find main()", file=sys.stderr)
        sys.exit(1)

    # Find all test_xxx() calls in main
    # Pattern: standalone "test_xxx();" or "test_bridge_xxx();"
    main_start = main_match.end()
    # Find the closing brace of main by counting braces
    depth = 1
    pos = main_start
    while pos < len(source) and depth > 0:
        if source[pos] == '{': depth += 1
        elif source[pos] == '}': depth -= 1
        pos += 1
    main_end = pos

    main_body = source[main_start:main_end-1]

    # Find test calls: "test_xxx();" at the start of a line (with optional whitespace)
    test_call_pattern = re.compile(r'^(\s+)(test_\w+)\(\);', re.MULTILINE)

    calls_found = test_call_pattern.findall(main_body)
    if not calls_found:
        print("WARNING: no test_xxx() calls found in main()", file=sys.stderr)
        with open(output_path, 'w') as f:
            f.write(source)
        return

    # Replace each "    test_xxx();" with "    run_test("xxx", test_xxx, TIMEOUT);"
    def replace_call(m):
        indent = m.group(1)
        func_name = m.group(2)
        # Extract short name (strip "test_" prefix for the label)
        short = func_name[5:] if func_name.startswith("test_") else func_name
        return f'{indent}run_test("{short}", {func_name}, {timeout_sec});'

    new_main_body = test_call_pattern.sub(replace_call, main_body)

    # Insert wrapper code before main()
    insert_pos = main_match.start()
    new_source = (
        source[:insert_pos]
        + WRAPPER_CODE + "\n"
        + source[insert_pos:main_start]
        + new_main_body
        + source[main_end-1:]
    )

    with open(output_path, 'w') as f:
        f.write(new_source)

    print(f"Wrapped {len(calls_found)} test calls with fork-based isolation (timeout={timeout_sec}s)")


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python3 wrap_tests_independent.py input.c output.c [timeout_sec]")
        sys.exit(1)
    timeout = int(sys.argv[3]) if len(sys.argv) > 3 else 5
    transform(sys.argv[1], sys.argv[2], timeout)
