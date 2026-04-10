#include "yaml.h"
#include <stdio.h>
#include <string.h>

/* Minimal smoke test: parse a simple YAML string, print events */
int main(void) {
    yaml_parser_t parser;
    yaml_event_t event;
    const char *input = "greeting: hello\nitems:\n  - one\n  - two\n";

    if (!yaml_parser_initialize(&parser)) {
        fprintf(stderr, "Failed to initialize parser\n");
        return 1;
    }
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));

    int done = 0;
    int count = 0;
    while (!done) {
        if (!yaml_parser_parse(&parser, &event)) {
            fprintf(stderr, "Parse error at line %lu\n",
                (unsigned long)parser.problem_mark.line);
            break;
        }
        printf("event %d: type=%d\n", count, event.type);
        done = (event.type == YAML_STREAM_END_EVENT);
        count++;
        yaml_event_delete(&event);
    }
    printf("Total events: %d\n", count);

    yaml_parser_delete(&parser);
    return 0;
}
