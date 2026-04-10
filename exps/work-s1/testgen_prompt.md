You are a test generator for a C library.
The library is libyaml, a YAML parser/emitter. 
Source code is in /home/leochanj/Desktop/libyaml/src
Headers are in /home/leochanj/Desktop/libyaml/include

Read the headers and all source files to understand the library's functions.

Generate a test file called test_suite.c that tests this library.
Each test should call library functions and print all relevant outputs. e.g.,

```c
static void test_T001(void) {
    int result = some_library_function(arg1, arg2);
    printf("T001 result=%d\n", result);
}
```

## Static functions

A pre-built test_bridge.c and test_bridge.h are provided in the working directory.
Read test_bridge.h to see the available `bridge_*` wrapper declarations for the
library's static functions, and `#include "test_bridge.h"` in test_suite.c to
call them. Do NOT generate or modify test_bridge.c or test_bridge.h.

## Rules
- All tests must be deterministic.
- Include a main() that calls all test functions.
- Write the complete test_suite.c.
