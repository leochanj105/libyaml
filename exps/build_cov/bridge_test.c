/* bridge_test.c — verify we can call static libyaml functions through bridges. */

#include "yaml.h"
#include <stdio.h>
#include <string.h>

/* Bridge wrappers (defined in build_cov/bridges/bridge_*.c) — we declare just
 * the ones this test calls. */
extern int bridge_yaml_check_utf8(const yaml_char_t *start, size_t length);
extern int bridge_yaml_parser_determine_encoding(yaml_parser_t *parser);
extern int bridge_yaml_emitter_check_empty_document(yaml_emitter_t *emitter);

int main(void) {
    /* 1. Static function from api.c: yaml_check_utf8
     *    Returns 1 if buffer is valid UTF-8. */
    const yaml_char_t valid[]   = "hello world";
    const yaml_char_t invalid[] = { 0xC0, 0xC1, 0x00 };
    printf("yaml_check_utf8(\"hello world\")    = %d (expect 1)\n",
           bridge_yaml_check_utf8(valid, strlen((const char *)valid)));
    printf("yaml_check_utf8(<invalid bytes>)   = %d (expect 0)\n",
           bridge_yaml_check_utf8(invalid, 2));

    /* 2. Static function from reader.c: yaml_parser_determine_encoding
     *    Needs an initialized parser with input set. */
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    const char *input = "hello: world\n";
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));
    int rc = bridge_yaml_parser_determine_encoding(&parser);
    printf("yaml_parser_determine_encoding()   = %d (expect 1)\n", rc);
    printf("  detected encoding = %d (1=UTF-8)\n", parser.encoding);
    yaml_parser_delete(&parser);

    /* 3. Static function from emitter.c: yaml_emitter_check_empty_document */
    yaml_emitter_t emitter;
    yaml_emitter_initialize(&emitter);
    int empty = bridge_yaml_emitter_check_empty_document(&emitter);
    printf("yaml_emitter_check_empty_document  = %d\n", empty);
    yaml_emitter_delete(&emitter);

    return 0;
}
