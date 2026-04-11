You are a test generator for a C library.

The library is libyaml, a YAML parser/emitter. Source code is in the C source tree directory provided.

Read ALL .c and .h files to understand every function in the library. Your goal is to generate tests that cover ALL functions, including internal/static ones, with a strong focus on **diverse corner cases**.

## Key modules to cover

1. **API** (api.c): yaml_parser_initialize, yaml_parser_delete, yaml_parser_set_input_string,
   yaml_emitter_initialize, yaml_emitter_delete, yaml_emitter_set_output_string,
   yaml_*_event_initialize, yaml_document_*, yaml_*_append, etc.

2. **Scanner** (scanner.c): tokenization — all token types (STREAM-START, BLOCK-MAPPING-START,
   FLOW-SEQUENCE-START, KEY, VALUE, SCALAR, TAG, ANCHOR, ALIAS, etc.)

3. **Parser** (parser.c): event parsing — STREAM-START, DOCUMENT-START, MAPPING-START,
   SEQUENCE-START, SCALAR, ALIAS, DOCUMENT-END, STREAM-END events.

4. **Emitter** (emitter.c): emit events back to YAML text — test round-trip (parse then emit).

5. **Reader** (reader.c): UTF-8, UTF-16LE, UTF-16BE encoding handling, BOM detection.

6. **Writer** (writer.c): output encoding.

7. **Loader/Dumper** (loader.c, dumper.c): high-level document API — yaml_parser_load,
   yaml_emitter_dump, building documents with nodes.

## Static/internal functions

Static functions cannot be called directly. Use the test bridge mechanism:
Create `test_bridge.c` that `#include`s the .c file and wraps the static function:

```c
#include "/absolute/path/to/scanner.c"
int bridge_yaml_scan_token(yaml_parser_t *p) {
    return yaml_parser_fetch_more_tokens(p);
}
```

Then in test_suite.c, declare: `extern int bridge_yaml_scan_token(yaml_parser_t *p);`

## Test structure — REQUIRED

Each test wrapped in #ifdef guard:

  #ifdef RUN_T001
  static void test_T001(void) {
      /* test body */
  }
  #endif

Called from main() inside the same guard:

  int main(void) {
  #ifdef RUN_T001
      test_T001();
  #endif
      return 0;
  }

## Coverage goals

- Test every public API function at least once
- Test error paths (NULL inputs, malformed YAML, memory allocation failures)
- Test different YAML features: anchors/aliases, tags, flow/block style,
  multi-document streams, Unicode, quoted scalars, literal/folded blocks
- Test edge cases: empty documents, deeply nested structures, very long scalars

## Expert corner cases — generate diverse tests specifically targeting these

The following are areas where YAML parsers commonly have subtle bugs. Generate
dedicated test cases for each of these:

### Scanner / Parser edge cases
- **Implicit vs explicit keys**: implicit keys have a length limit (1024 chars);
  test keys near and beyond this limit. Test explicit keys (`? key`) in both
  block and flow contexts.
- **Block scalar chomping**: literal (`|`) and folded (`>`) scalars with all
  three chomping indicators: clip (default), strip (`|-`), keep (`|+`).
  Test with trailing newlines, no trailing newline, and multiple trailing newlines.
- **Flow collection nesting**: deeply nested flow sequences `[[[[...]]]]` and
  mixed flow/block contexts. Test the scanner's flow_level transitions.
- **Anchor/alias resolution**: aliases to anchors defined later in the stream
  (forward references are invalid), duplicate anchor names, aliases in nested
  structures, anchor on empty nodes.
- **Tag resolution**: verbatim tags (`!<tag:yaml.org,2002:str>`), shorthand tags
  (`!!str`, `!foo`), tag handles (`%TAG !e! tag:example.com,2000:`), and
  invalid/unknown tag handles.
- **Multi-document streams**: `---` and `...` markers, documents with and without
  explicit markers, bare documents, empty documents between markers.
- **Indentation edge cases**: minimal indentation (1 space), inconsistent
  indentation within a block (should error), zero-indentation after directive.

### Encoding edge cases
- **BOM handling**: UTF-8 BOM (EF BB BF), UTF-16LE BOM (FF FE), UTF-16BE BOM
  (FE FF), no BOM (defaults to UTF-8). Test that BOM is consumed and not
  emitted in output.
- **Multi-byte UTF-8**: 2-byte (U+0080–U+07FF), 3-byte (U+0800–U+FFFF),
  4-byte (U+10000–U+10FFFF) characters in scalars, keys, and comments.
- **Invalid UTF-8 sequences**: overlong encodings, truncated sequences,
  invalid continuation bytes — parser should reject these.
- **NUL bytes**: NUL (0x00) in the middle of a stream is a valid YAML break
  indicator in some contexts but an error in others.

### Emitter edge cases
- **Style selection**: force plain, single-quoted, double-quoted, literal, folded
  styles via yaml_scalar_event_initialize. Test that the emitter picks the
  right quoting when style is YAML_ANY_SCALAR_STYLE — especially for strings
  containing `: `, `# `, `[`, `]`, `{`, `}`, `\n`, `\t`.
- **Line breaking**: emitter should break long lines. Test with
  yaml_emitter_set_width() at various values (1, 40, 80, very large).
- **Canonical mode**: yaml_emitter_set_canonical — forces explicit tags and
  verbose output.
- **Unicode escaping**: emitter should escape control characters (U+0000–U+001F
  except TAB/LF/CR) in double-quoted scalars.

### Document API edge cases
- **Circular references**: creating a document with a sequence that contains an
  alias to itself (should be representable in the node graph).
- **Large node IDs**: documents with many nodes (hundreds) to stress the
  node array reallocation.
- **Empty mapping/sequence nodes**: nodes with zero children.
- **Delete partially built documents**: call yaml_document_delete on a document
  that has nodes added but was never dumped.
