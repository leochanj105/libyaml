/*
 * test_suite.c — libyaml test suite
 * Each test calls library functions and prints relevant outputs.
 */

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "yaml.h"
#include "test_bridge.h"

/* -----------------------------------------------------------------------
 * T001: yaml_get_version_string / yaml_get_version
 * --------------------------------------------------------------------- */
static void test_T001(void) {
    const char *ver = yaml_get_version_string();
    int major = 0, minor = 0, patch = 0;
    yaml_get_version(&major, &minor, &patch);
    printf("T001 version_string=%s major=%d minor=%d patch=%d\n",
           ver, major, minor, patch);
}

/* -----------------------------------------------------------------------
 * T002: yaml_malloc / yaml_realloc / yaml_free
 * --------------------------------------------------------------------- */
static void test_T002(void) {
    void *p = yaml_malloc(64);
    printf("T002 malloc_ok=%d\n", p != NULL);
    p = yaml_realloc(p, 128);
    printf("T002 realloc_ok=%d\n", p != NULL);
    yaml_free(p);
    printf("T002 free_ok=1\n");
    /* yaml_free(NULL) must be safe */
    yaml_free(NULL);
    printf("T002 free_null_ok=1\n");
}

/* -----------------------------------------------------------------------
 * T003: yaml_strdup
 * --------------------------------------------------------------------- */
static void test_T003(void) {
    yaml_char_t *dup = yaml_strdup((const yaml_char_t *)"hello");
    printf("T003 strdup=%s\n", (char *)dup);
    yaml_free(dup);
    yaml_char_t *null_dup = yaml_strdup(NULL);
    printf("T003 strdup_null=%d\n", null_dup == NULL);
}

/* -----------------------------------------------------------------------
 * T004: bridge_yaml_check_utf8 — valid and invalid UTF-8
 * --------------------------------------------------------------------- */
static void test_T004(void) {
    const yaml_char_t valid[]   = (const yaml_char_t *)"hello world";
    /* 0xFF is not valid UTF-8 */
    const yaml_char_t invalid[] = { 0xFF, 0x00 };

    int r1 = bridge_yaml_check_utf8(valid,   strlen((char *)valid));
    int r2 = bridge_yaml_check_utf8(invalid, 1);
    printf("T004 valid_utf8=%d invalid_utf8=%d\n", r1, r2);
}

/* -----------------------------------------------------------------------
 * T005: yaml_token_delete (no crash on zeroed token)
 * --------------------------------------------------------------------- */
static void test_T005(void) {
    yaml_token_t tok;
    memset(&tok, 0, sizeof(tok));
    yaml_token_delete(&tok);
    printf("T005 token_delete_ok=1\n");
}

/* -----------------------------------------------------------------------
 * T006: yaml_event_delete (no crash on zeroed event)
 * --------------------------------------------------------------------- */
static void test_T006(void) {
    yaml_event_t ev;
    memset(&ev, 0, sizeof(ev));
    yaml_event_delete(&ev);
    printf("T006 event_delete_ok=1\n");
}

/* -----------------------------------------------------------------------
 * T007: event initializers
 * --------------------------------------------------------------------- */
static void test_T007(void) {
    yaml_event_t ev;

    int r1 = yaml_stream_start_event_initialize(&ev, YAML_UTF8_ENCODING);
    printf("T007 stream_start_init=%d type=%d\n", r1, ev.type);
    yaml_event_delete(&ev);

    int r2 = yaml_stream_end_event_initialize(&ev);
    printf("T007 stream_end_init=%d type=%d\n", r2, ev.type);
    yaml_event_delete(&ev);

    int r3 = yaml_document_start_event_initialize(&ev, NULL, NULL, NULL, 1);
    printf("T007 doc_start_init=%d type=%d implicit=%d\n",
           r3, ev.type, ev.data.document_start.implicit);
    yaml_event_delete(&ev);

    int r4 = yaml_document_end_event_initialize(&ev, 1);
    printf("T007 doc_end_init=%d type=%d implicit=%d\n",
           r4, ev.type, ev.data.document_end.implicit);
    yaml_event_delete(&ev);

    int r5 = yaml_alias_event_initialize(&ev, (const yaml_char_t *)"myanchor");
    printf("T007 alias_init=%d type=%d anchor=%s\n",
           r5, ev.type, (char *)ev.data.alias.anchor);
    yaml_event_delete(&ev);

    int r6 = yaml_scalar_event_initialize(&ev, NULL, NULL,
                (const yaml_char_t *)"value", 5, 1, 0,
                YAML_PLAIN_SCALAR_STYLE);
    printf("T007 scalar_init=%d type=%d value=%s\n",
           r6, ev.type, (char *)ev.data.scalar.value);
    yaml_event_delete(&ev);

    int r7 = yaml_sequence_start_event_initialize(&ev, NULL, NULL, 1,
                YAML_BLOCK_SEQUENCE_STYLE);
    printf("T007 seq_start_init=%d type=%d\n", r7, ev.type);
    yaml_event_delete(&ev);

    int r8 = yaml_sequence_end_event_initialize(&ev);
    printf("T007 seq_end_init=%d type=%d\n", r8, ev.type);
    yaml_event_delete(&ev);

    int r9 = yaml_mapping_start_event_initialize(&ev, NULL, NULL, 1,
                YAML_BLOCK_MAPPING_STYLE);
    printf("T007 map_start_init=%d type=%d\n", r9, ev.type);
    yaml_event_delete(&ev);

    int r10 = yaml_mapping_end_event_initialize(&ev);
    printf("T007 map_end_init=%d type=%d\n", r10, ev.type);
    yaml_event_delete(&ev);
}

/* -----------------------------------------------------------------------
 * T008: yaml_document_initialize / yaml_document_delete
 * --------------------------------------------------------------------- */
static void test_T008(void) {
    yaml_document_t doc;
    int r = yaml_document_initialize(&doc, NULL, NULL, NULL, 1, 1);
    printf("T008 doc_init=%d start_implicit=%d end_implicit=%d\n",
           r, doc.start_implicit, doc.end_implicit);
    yaml_document_delete(&doc);
    printf("T008 doc_delete_ok=1\n");
}

/* -----------------------------------------------------------------------
 * T009: yaml_document_add_scalar / get_node / get_root_node
 * --------------------------------------------------------------------- */
static void test_T009(void) {
    yaml_document_t doc;
    yaml_document_initialize(&doc, NULL, NULL, NULL, 1, 1);

    int id = yaml_document_add_scalar(&doc,
                (const yaml_char_t *)YAML_STR_TAG,
                (const yaml_char_t *)"hello", 5,
                YAML_PLAIN_SCALAR_STYLE);
    printf("T009 scalar_id=%d\n", id);

    yaml_node_t *node = yaml_document_get_node(&doc, id);
    printf("T009 node_type=%d value=%s\n",
           node->type, (char *)node->data.scalar.value);

    yaml_node_t *root = yaml_document_get_root_node(&doc);
    printf("T009 root_type=%d\n", root->type);

    yaml_document_delete(&doc);
}

/* -----------------------------------------------------------------------
 * T010: yaml_document_add_sequence / append_sequence_item
 * --------------------------------------------------------------------- */
static void test_T010(void) {
    yaml_document_t doc;
    yaml_document_initialize(&doc, NULL, NULL, NULL, 1, 1);

    int seq_id = yaml_document_add_sequence(&doc,
                    (const yaml_char_t *)YAML_SEQ_TAG,
                    YAML_BLOCK_SEQUENCE_STYLE);
    printf("T010 seq_id=%d\n", seq_id);

    int s1 = yaml_document_add_scalar(&doc, NULL,
                (const yaml_char_t *)"item1", 5, YAML_PLAIN_SCALAR_STYLE);
    int s2 = yaml_document_add_scalar(&doc, NULL,
                (const yaml_char_t *)"item2", 5, YAML_PLAIN_SCALAR_STYLE);

    int r1 = yaml_document_append_sequence_item(&doc, seq_id, s1);
    int r2 = yaml_document_append_sequence_item(&doc, seq_id, s2);
    printf("T010 append_item1=%d append_item2=%d\n", r1, r2);

    yaml_node_t *node = yaml_document_get_node(&doc, seq_id);
    int count = (int)(node->data.sequence.items.top -
                      node->data.sequence.items.start);
    printf("T010 item_count=%d\n", count);

    yaml_document_delete(&doc);
}

/* -----------------------------------------------------------------------
 * T011: yaml_document_add_mapping / append_mapping_pair
 * --------------------------------------------------------------------- */
static void test_T011(void) {
    yaml_document_t doc;
    yaml_document_initialize(&doc, NULL, NULL, NULL, 1, 1);

    int map_id = yaml_document_add_mapping(&doc,
                    (const yaml_char_t *)YAML_MAP_TAG,
                    YAML_BLOCK_MAPPING_STYLE);
    printf("T011 map_id=%d\n", map_id);

    int k = yaml_document_add_scalar(&doc, NULL,
                (const yaml_char_t *)"key", 3, YAML_PLAIN_SCALAR_STYLE);
    int v = yaml_document_add_scalar(&doc, NULL,
                (const yaml_char_t *)"val", 3, YAML_PLAIN_SCALAR_STYLE);

    int r = yaml_document_append_mapping_pair(&doc, map_id, k, v);
    printf("T011 append_pair=%d\n", r);

    yaml_node_t *node = yaml_document_get_node(&doc, map_id);
    int count = (int)(node->data.mapping.pairs.top -
                      node->data.mapping.pairs.start);
    printf("T011 pair_count=%d\n", count);

    yaml_document_delete(&doc);
}

/* -----------------------------------------------------------------------
 * T012: yaml_parser_initialize / yaml_parser_delete
 * --------------------------------------------------------------------- */
static void test_T012(void) {
    yaml_parser_t parser;
    int r = yaml_parser_initialize(&parser);
    printf("T012 parser_init=%d\n", r);
    yaml_parser_delete(&parser);
    printf("T012 parser_delete_ok=1\n");
}

/* -----------------------------------------------------------------------
 * T013: yaml_parser_scan — simple scalar YAML
 * --------------------------------------------------------------------- */
static void test_T013(void) {
    const char *input = "hello\n";
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));

    int token_count = 0;
    yaml_token_t token;
    while (1) {
        if (!yaml_parser_scan(&parser, &token)) {
            printf("T013 scan_error=%s\n", parser.problem);
            yaml_token_delete(&token);
            break;
        }
        token_count++;
        int done = (token.type == YAML_STREAM_END_TOKEN);
        yaml_token_delete(&token);
        if (done) break;
    }
    printf("T013 token_count=%d\n", token_count);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T014: yaml_parser_parse — event-based parsing of simple scalar
 * --------------------------------------------------------------------- */
static void test_T014(void) {
    const char *input = "hello\n";
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));

    int event_count = 0;
    yaml_event_t event;
    while (1) {
        if (!yaml_parser_parse(&parser, &event)) {
            printf("T014 parse_error=%s\n", parser.problem);
            yaml_event_delete(&event);
            break;
        }
        event_count++;
        int done = (event.type == YAML_STREAM_END_EVENT);
        yaml_event_delete(&event);
        if (done) break;
    }
    printf("T014 event_count=%d\n", event_count);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T015: yaml_parser_load — load document from string
 * --------------------------------------------------------------------- */
static void test_T015(void) {
    const char *input = "key: value\n";
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));

    yaml_document_t doc;
    int r = yaml_parser_load(&parser, &doc);
    printf("T015 load=%d\n", r);

    yaml_node_t *root = yaml_document_get_root_node(&doc);
    printf("T015 root_type=%d\n", root ? (int)root->type : -1);

    yaml_document_delete(&doc);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T016: yaml_parser_set_encoding
 * --------------------------------------------------------------------- */
static void test_T016(void) {
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_parser_set_encoding(&parser, YAML_UTF8_ENCODING);
    printf("T016 encoding=%d\n", (int)parser.encoding);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T017: yaml_emitter_initialize / delete
 * --------------------------------------------------------------------- */
static void test_T017(void) {
    yaml_emitter_t emitter;
    int r = yaml_emitter_initialize(&emitter);
    printf("T017 emitter_init=%d\n", r);
    yaml_emitter_delete(&emitter);
    printf("T017 emitter_delete_ok=1\n");
}

/* -----------------------------------------------------------------------
 * T018: emit simple scalar YAML to string buffer
 * --------------------------------------------------------------------- */
static void test_T018(void) {
    unsigned char buf[512];
    size_t written = 0;

    yaml_emitter_t emitter;
    yaml_emitter_initialize(&emitter);
    yaml_emitter_set_output_string(&emitter, buf, sizeof(buf), &written);

    yaml_event_t ev;
    yaml_stream_start_event_initialize(&ev, YAML_UTF8_ENCODING);
    yaml_emitter_emit(&emitter, &ev);

    yaml_document_start_event_initialize(&ev, NULL, NULL, NULL, 1);
    yaml_emitter_emit(&emitter, &ev);

    yaml_scalar_event_initialize(&ev, NULL, NULL,
        (const yaml_char_t *)"hello", 5, 1, 0, YAML_PLAIN_SCALAR_STYLE);
    yaml_emitter_emit(&emitter, &ev);

    yaml_document_end_event_initialize(&ev, 1);
    yaml_emitter_emit(&emitter, &ev);

    yaml_stream_end_event_initialize(&ev);
    yaml_emitter_emit(&emitter, &ev);

    yaml_emitter_flush(&emitter);
    buf[written] = '\0';
    printf("T018 written=%zu output=%s\n", written, (char *)buf);
    yaml_emitter_delete(&emitter);
}

/* -----------------------------------------------------------------------
 * T019: emitter settings (canonical, indent, width, unicode, break)
 * --------------------------------------------------------------------- */
static void test_T019(void) {
    yaml_emitter_t emitter;
    yaml_emitter_initialize(&emitter);

    yaml_emitter_set_canonical(&emitter, 1);
    yaml_emitter_set_indent(&emitter, 4);
    yaml_emitter_set_width(&emitter, 80);
    yaml_emitter_set_unicode(&emitter, 1);
    yaml_emitter_set_break(&emitter, YAML_LN_BREAK);

    printf("T019 canonical=%d indent=%d width=%d unicode=%d break=%d\n",
           emitter.canonical, emitter.best_indent,
           emitter.best_width, emitter.unicode, (int)emitter.line_break);

    yaml_emitter_delete(&emitter);
}

/* -----------------------------------------------------------------------
 * T020: yaml_emitter_open / close (dumper API)
 * --------------------------------------------------------------------- */
static void test_T020(void) {
    unsigned char buf[256];
    size_t written = 0;

    yaml_emitter_t emitter;
    yaml_emitter_initialize(&emitter);
    yaml_emitter_set_output_string(&emitter, buf, sizeof(buf), &written);

    int r1 = yaml_emitter_open(&emitter);
    int r2 = yaml_emitter_close(&emitter);
    yaml_emitter_flush(&emitter);
    buf[written] = '\0';
    printf("T020 open=%d close=%d written=%zu output=%s\n",
           r1, r2, written, (char *)buf);
    yaml_emitter_delete(&emitter);
}

/* -----------------------------------------------------------------------
 * T021: yaml_emitter_dump — dump a document via the high-level API
 * --------------------------------------------------------------------- */
static void test_T021(void) {
    unsigned char buf[512];
    size_t written = 0;

    yaml_emitter_t emitter;
    yaml_emitter_initialize(&emitter);
    yaml_emitter_set_output_string(&emitter, buf, sizeof(buf), &written);
    yaml_emitter_open(&emitter);

    yaml_document_t doc;
    yaml_document_initialize(&doc, NULL, NULL, NULL, 1, 1);
    int scalar_id = yaml_document_add_scalar(&doc,
        (const yaml_char_t *)YAML_STR_TAG,
        (const yaml_char_t *)"world", 5, YAML_PLAIN_SCALAR_STYLE);
    printf("T021 scalar_id=%d\n", scalar_id);

    int r = yaml_emitter_dump(&emitter, &doc);
    yaml_emitter_close(&emitter);
    yaml_emitter_flush(&emitter);
    buf[written] = '\0';
    printf("T021 dump=%d written=%zu output=%s\n", r, written, (char *)buf);
    yaml_emitter_delete(&emitter);
}

/* -----------------------------------------------------------------------
 * T022: yaml_parser_load — mapping document, check node types
 * --------------------------------------------------------------------- */
static void test_T022(void) {
    const char *input = "name: Alice\nage: 30\n";
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));

    yaml_document_t doc;
    yaml_parser_load(&parser, &doc);

    yaml_node_t *root = yaml_document_get_root_node(&doc);
    printf("T022 root_type=%d\n", root ? (int)root->type : -1);

    if (root && root->type == YAML_MAPPING_NODE) {
        int pair_count = (int)(root->data.mapping.pairs.top -
                               root->data.mapping.pairs.start);
        printf("T022 pair_count=%d\n", pair_count);
        for (int i = 0; i < pair_count; i++) {
            yaml_node_pair_t *pair = root->data.mapping.pairs.start + i;
            yaml_node_t *key = yaml_document_get_node(&doc, pair->key);
            yaml_node_t *val = yaml_document_get_node(&doc, pair->value);
            printf("T022 pair[%d] key=%s val=%s\n", i,
                   (char *)key->data.scalar.value,
                   (char *)val->data.scalar.value);
        }
    }

    yaml_document_delete(&doc);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T023: yaml_parser_load — sequence document
 * --------------------------------------------------------------------- */
static void test_T023(void) {
    const char *input = "- a\n- b\n- c\n";
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));

    yaml_document_t doc;
    yaml_parser_load(&parser, &doc);

    yaml_node_t *root = yaml_document_get_root_node(&doc);
    printf("T023 root_type=%d\n", root ? (int)root->type : -1);

    if (root && root->type == YAML_SEQUENCE_NODE) {
        int item_count = (int)(root->data.sequence.items.top -
                               root->data.sequence.items.start);
        printf("T023 item_count=%d\n", item_count);
        for (int i = 0; i < item_count; i++) {
            int idx = root->data.sequence.items.start[i];
            yaml_node_t *node = yaml_document_get_node(&doc, idx);
            printf("T023 item[%d]=%s\n", i, (char *)node->data.scalar.value);
        }
    }

    yaml_document_delete(&doc);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T024: yaml_parser_scan — count tokens for block mapping
 * --------------------------------------------------------------------- */
static void test_T024(void) {
    const char *input = "x: 1\ny: 2\n";
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));

    int token_count = 0;
    yaml_token_t token;
    while (1) {
        if (!yaml_parser_scan(&parser, &token)) break;
        token_count++;
        int done = (token.type == YAML_STREAM_END_TOKEN);
        yaml_token_delete(&token);
        if (done) break;
    }
    printf("T024 token_count=%d\n", token_count);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T025: parse invalid YAML — expect error
 * --------------------------------------------------------------------- */
static void test_T025(void) {
    const char *input = "key: [\n";  /* unclosed flow sequence */
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));

    yaml_event_t event;
    int got_error = 0;
    while (1) {
        if (!yaml_parser_parse(&parser, &event)) {
            got_error = 1;
            printf("T025 error_type=%d\n", (int)parser.error);
            break;
        }
        int done = (event.type == YAML_STREAM_END_EVENT);
        yaml_event_delete(&event);
        if (done) break;
    }
    printf("T025 got_error=%d\n", got_error);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T026: bridge_yaml_string_read_handler
 * --------------------------------------------------------------------- */
static void test_T026(void) {
    /* The string read handler reads from parser->input.string */
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    const char *src = "abcde";
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)src, strlen(src));

    unsigned char buf[8];
    size_t size_read = 0;
    int r = bridge_yaml_string_read_handler(&parser, buf, sizeof(buf), &size_read);
    printf("T026 read_ok=%d size_read=%zu\n", r, size_read);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T027: bridge_yaml_emitter_set_emitter_error
 * --------------------------------------------------------------------- */
static void test_T027(void) {
    yaml_emitter_t emitter;
    yaml_emitter_initialize(&emitter);
    int r = bridge_yaml_emitter_set_emitter_error(&emitter, "test error");
    printf("T027 result=%d error_type=%d problem=%s\n",
           r, (int)emitter.error, emitter.problem);
    yaml_emitter_delete(&emitter);
}

/* -----------------------------------------------------------------------
 * T028: bridge_yaml_parser_set_reader_error
 * --------------------------------------------------------------------- */
static void test_T028(void) {
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    int r = bridge_yaml_parser_set_reader_error(&parser, "bad byte", 5, 0xFF);
    printf("T028 result=%d error_type=%d problem=%s offset=%zu\n",
           r, (int)parser.error, parser.problem, parser.problem_offset);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T029: bridge_yaml_parser_set_scanner_error
 * --------------------------------------------------------------------- */
static void test_T029(void) {
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_mark_t mark = {0, 0, 0};
    int r = bridge_yaml_parser_set_scanner_error(
                &parser, "context", mark, "problem");
    printf("T029 result=%d error_type=%d problem=%s\n",
           r, (int)parser.error, parser.problem);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T030: yaml_set_max_nest_level — parse deeply nested but within limit
 * --------------------------------------------------------------------- */
static void test_T030(void) {
    yaml_set_max_nest_level(10);
    /* Build a YAML string with 5 levels of flow sequences */
    const char *input = "[[[[[ ]]]]]\n";
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));

    int event_count = 0;
    yaml_event_t event;
    int ok = 1;
    while (1) {
        if (!yaml_parser_parse(&parser, &event)) { ok = 0; break; }
        event_count++;
        int done = (event.type == YAML_STREAM_END_EVENT);
        yaml_event_delete(&event);
        if (done) break;
    }
    printf("T030 ok=%d event_count=%d\n", ok, event_count);
    yaml_parser_delete(&parser);
    /* reset to default */
    yaml_set_max_nest_level(1000);
}

/* -----------------------------------------------------------------------
 * T031: emitter — flow sequence output
 * --------------------------------------------------------------------- */
static void test_T031(void) {
    unsigned char buf[256];
    size_t written = 0;

    yaml_emitter_t emitter;
    yaml_emitter_initialize(&emitter);
    yaml_emitter_set_output_string(&emitter, buf, sizeof(buf), &written);

    yaml_event_t ev;
    yaml_stream_start_event_initialize(&ev, YAML_UTF8_ENCODING);
    yaml_emitter_emit(&emitter, &ev);

    yaml_document_start_event_initialize(&ev, NULL, NULL, NULL, 1);
    yaml_emitter_emit(&emitter, &ev);

    yaml_sequence_start_event_initialize(&ev, NULL, NULL, 1,
        YAML_FLOW_SEQUENCE_STYLE);
    yaml_emitter_emit(&emitter, &ev);

    yaml_scalar_event_initialize(&ev, NULL, NULL,
        (const yaml_char_t *)"x", 1, 1, 0, YAML_PLAIN_SCALAR_STYLE);
    yaml_emitter_emit(&emitter, &ev);

    yaml_scalar_event_initialize(&ev, NULL, NULL,
        (const yaml_char_t *)"y", 1, 1, 0, YAML_PLAIN_SCALAR_STYLE);
    yaml_emitter_emit(&emitter, &ev);

    yaml_sequence_end_event_initialize(&ev);
    yaml_emitter_emit(&emitter, &ev);

    yaml_document_end_event_initialize(&ev, 1);
    yaml_emitter_emit(&emitter, &ev);

    yaml_stream_end_event_initialize(&ev);
    yaml_emitter_emit(&emitter, &ev);

    yaml_emitter_flush(&emitter);
    buf[written] = '\0';
    printf("T031 written=%zu output=%s\n", written, (char *)buf);
    yaml_emitter_delete(&emitter);
}

/* -----------------------------------------------------------------------
 * T032: emitter — flow mapping output
 * --------------------------------------------------------------------- */
static void test_T032(void) {
    unsigned char buf[256];
    size_t written = 0;

    yaml_emitter_t emitter;
    yaml_emitter_initialize(&emitter);
    yaml_emitter_set_output_string(&emitter, buf, sizeof(buf), &written);

    yaml_event_t ev;
    yaml_stream_start_event_initialize(&ev, YAML_UTF8_ENCODING);
    yaml_emitter_emit(&emitter, &ev);

    yaml_document_start_event_initialize(&ev, NULL, NULL, NULL, 1);
    yaml_emitter_emit(&emitter, &ev);

    yaml_mapping_start_event_initialize(&ev, NULL, NULL, 1,
        YAML_FLOW_MAPPING_STYLE);
    yaml_emitter_emit(&emitter, &ev);

    yaml_scalar_event_initialize(&ev, NULL, NULL,
        (const yaml_char_t *)"k", 1, 1, 0, YAML_PLAIN_SCALAR_STYLE);
    yaml_emitter_emit(&emitter, &ev);

    yaml_scalar_event_initialize(&ev, NULL, NULL,
        (const yaml_char_t *)"v", 1, 1, 0, YAML_PLAIN_SCALAR_STYLE);
    yaml_emitter_emit(&emitter, &ev);

    yaml_mapping_end_event_initialize(&ev);
    yaml_emitter_emit(&emitter, &ev);

    yaml_document_end_event_initialize(&ev, 1);
    yaml_emitter_emit(&emitter, &ev);

    yaml_stream_end_event_initialize(&ev);
    yaml_emitter_emit(&emitter, &ev);

    yaml_emitter_flush(&emitter);
    buf[written] = '\0';
    printf("T032 written=%zu output=%s\n", written, (char *)buf);
    yaml_emitter_delete(&emitter);
}

/* -----------------------------------------------------------------------
 * T033: yaml_document_get_node with out-of-range index
 * --------------------------------------------------------------------- */
static void test_T033(void) {
    yaml_document_t doc;
    yaml_document_initialize(&doc, NULL, NULL, NULL, 1, 1);
    yaml_document_add_scalar(&doc, NULL,
        (const yaml_char_t *)"x", 1, YAML_PLAIN_SCALAR_STYLE);

    yaml_node_t *bad = yaml_document_get_node(&doc, 99);
    printf("T033 out_of_range_node=%d\n", bad == NULL);
    yaml_node_t *zero = yaml_document_get_node(&doc, 0);
    printf("T033 zero_index_node=%d\n", zero == NULL);

    yaml_document_delete(&doc);
}

/* -----------------------------------------------------------------------
 * T034: yaml_document_get_root_node on empty document
 * --------------------------------------------------------------------- */
static void test_T034(void) {
    yaml_document_t doc;
    yaml_document_initialize(&doc, NULL, NULL, NULL, 1, 1);
    yaml_node_t *root = yaml_document_get_root_node(&doc);
    printf("T034 empty_root=%d\n", root == NULL);
    yaml_document_delete(&doc);
}

/* -----------------------------------------------------------------------
 * T035: bridge_yaml_check_utf8 — empty string
 * --------------------------------------------------------------------- */
static void test_T035(void) {
    int r = bridge_yaml_check_utf8((const yaml_char_t *)"", 0);
    printf("T035 empty_utf8=%d\n", r);
}

/* -----------------------------------------------------------------------
 * T036: parse YAML with anchor and alias
 * --------------------------------------------------------------------- */
static void test_T036(void) {
    const char *input = "- &anchor value\n- *anchor\n";
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));

    yaml_document_t doc;
    int r = yaml_parser_load(&parser, &doc);
    printf("T036 load=%d\n", r);

    yaml_node_t *root = yaml_document_get_root_node(&doc);
    if (root && root->type == YAML_SEQUENCE_NODE) {
        int count = (int)(root->data.sequence.items.top -
                          root->data.sequence.items.start);
        printf("T036 item_count=%d\n", count);
    }

    yaml_document_delete(&doc);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T037: parse multi-document YAML stream
 * --------------------------------------------------------------------- */
static void test_T037(void) {
    const char *input = "---\nfoo\n---\nbar\n";
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_parser_set_input_string(&parser,
        (const unsigned char *)input, strlen(input));

    int doc_count = 0;
    while (1) {
        yaml_document_t doc;
        if (!yaml_parser_load(&parser, &doc)) break;
        yaml_node_t *root = yaml_document_get_root_node(&doc);
        if (!root) { yaml_document_delete(&doc); break; }
        doc_count++;
        printf("T037 doc[%d] root_type=%d value=%s\n",
               doc_count, (int)root->type,
               root->type == YAML_SCALAR_NODE ?
                   (char *)root->data.scalar.value : "");
        yaml_document_delete(&doc);
    }
    printf("T037 doc_count=%d\n", doc_count);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * T038: emitter — double-quoted scalar style
 * --------------------------------------------------------------------- */
static void test_T038(void) {
    unsigned char buf[256];
    size_t written = 0;

    yaml_emitter_t emitter;
    yaml_emitter_initialize(&emitter);
    yaml_emitter_set_output_string(&emitter, buf, sizeof(buf), &written);

    yaml_event_t ev;
    yaml_stream_start_event_initialize(&ev, YAML_UTF8_ENCODING);
    yaml_emitter_emit(&emitter, &ev);

    yaml_document_start_event_initialize(&ev, NULL, NULL, NULL, 1);
    yaml_emitter_emit(&emitter, &ev);

    yaml_scalar_event_initialize(&ev, NULL, NULL,
        (const yaml_char_t *)"hello world", 11,
        0, 1, YAML_DOUBLE_QUOTED_SCALAR_STYLE);
    yaml_emitter_emit(&emitter, &ev);

    yaml_document_end_event_initialize(&ev, 1);
    yaml_emitter_emit(&emitter, &ev);

    yaml_stream_end_event_initialize(&ev);
    yaml_emitter_emit(&emitter, &ev);

    yaml_emitter_flush(&emitter);
    buf[written] = '\0';
    printf("T038 output=%s\n", (char *)buf);
    yaml_emitter_delete(&emitter);
}

/* -----------------------------------------------------------------------
 * T039: bridge_yaml_emitter_set_writer_error
 * --------------------------------------------------------------------- */
static void test_T039(void) {
    yaml_emitter_t emitter;
    yaml_emitter_initialize(&emitter);
    int r = bridge_yaml_emitter_set_writer_error(&emitter, "write failed");
    printf("T039 result=%d error_type=%d problem=%s\n",
           r, (int)emitter.error, emitter.problem);
    yaml_emitter_delete(&emitter);
}

/* -----------------------------------------------------------------------
 * T040: bridge_yaml_parser_set_parser_error
 * --------------------------------------------------------------------- */
static void test_T040(void) {
    yaml_parser_t parser;
    yaml_parser_initialize(&parser);
    yaml_mark_t mark = {3, 1, 2};
    int r = bridge_yaml_parser_set_parser_error(&parser, "bad token", mark);
    printf("T040 result=%d error_type=%d problem=%s\n",
           r, (int)parser.error, parser.problem);
    yaml_parser_delete(&parser);
}

/* -----------------------------------------------------------------------
 * main
 * --------------------------------------------------------------------- */
int main(void) {
    test_T001();
    test_T002();
    test_T003();
    test_T004();
    test_T005();
    test_T006();
    test_T007();
    test_T008();
    test_T009();
    test_T010();
    test_T011();
    test_T012();
    test_T013();
    test_T014();
    test_T015();
    test_T016();
    test_T017();
    test_T018();
    test_T019();
    test_T020();
    test_T021();
    test_T022();
    test_T023();
    test_T024();
    test_T025();
    test_T026();
    test_T027();
    test_T028();
    test_T029();
    test_T030();
    test_T031();
    test_T032();
    test_T033();
    test_T034();
    test_T035();
    test_T036();
    test_T037();
    test_T038();
    test_T039();
    test_T040();
    return 0;
}
