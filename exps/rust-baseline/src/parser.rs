use crate::*;
use libc::{c_char, c_int, size_t};

// Forward declaration for scanner function defined in scanner.rs
// (yaml_parser_fetch_more_tokens is defined in crate::scanner)
use crate::scanner::yaml_parser_fetch_more_tokens;

pub static mut MAX_NESTING_LEVEL: c_int = 1000;

// Parser-specific helper: PEEK_TOKEN
// Returns a pointer to the head token, fetching more if needed.
unsafe fn peek_token(parser: *mut yaml_parser_t) -> *mut yaml_token_t {
    if (*parser).token_available != 0 || yaml_parser_fetch_more_tokens(parser) != 0 {
        (*parser).tokens.head
    } else {
        core::ptr::null_mut()
    }
}

// Parser-specific helper: SKIP_TOKEN
unsafe fn skip_token(parser: *mut yaml_parser_t) {
    (*parser).token_available = 0;
    (*parser).tokens_parsed += 1;
    (*parser).stream_end_produced =
        ((*(*parser).tokens.head).type_ == YAML_STREAM_END_TOKEN) as c_int;
    (*parser).tokens.head = (*parser).tokens.head.add(1);
}

// Event init macros (local to parser)

macro_rules! STREAM_START_EVENT_INIT {
    ($event:expr, $encoding:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $event as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_event_t>(),
        );
        $event.type_ = YAML_STREAM_START_EVENT;
        $event.start_mark = $start_mark;
        $event.end_mark = $end_mark;
        $event.data.stream_start.encoding = $encoding;
    }};
}

macro_rules! STREAM_END_EVENT_INIT {
    ($event:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $event as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_event_t>(),
        );
        $event.type_ = YAML_STREAM_END_EVENT;
        $event.start_mark = $start_mark;
        $event.end_mark = $end_mark;
    }};
}

macro_rules! DOCUMENT_START_EVENT_INIT {
    ($event:expr, $version_directive:expr, $tag_directives_start:expr, $tag_directives_end:expr, $implicit:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $event as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_event_t>(),
        );
        $event.type_ = YAML_DOCUMENT_START_EVENT;
        $event.start_mark = $start_mark;
        $event.end_mark = $end_mark;
        $event.data.document_start.version_directive = $version_directive;
        $event.data.document_start.tag_directives.start = $tag_directives_start;
        $event.data.document_start.tag_directives.end = $tag_directives_end;
        $event.data.document_start.implicit = $implicit;
    }};
}

macro_rules! DOCUMENT_END_EVENT_INIT {
    ($event:expr, $implicit:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $event as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_event_t>(),
        );
        $event.type_ = YAML_DOCUMENT_END_EVENT;
        $event.start_mark = $start_mark;
        $event.end_mark = $end_mark;
        $event.data.document_end.implicit = $implicit;
    }};
}

macro_rules! ALIAS_EVENT_INIT {
    ($event:expr, $anchor:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $event as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_event_t>(),
        );
        $event.type_ = YAML_ALIAS_EVENT;
        $event.start_mark = $start_mark;
        $event.end_mark = $end_mark;
        $event.data.alias.anchor = $anchor;
    }};
}

macro_rules! SCALAR_EVENT_INIT {
    ($event:expr, $anchor:expr, $tag:expr, $value:expr, $length:expr, $plain_implicit:expr, $quoted_implicit:expr, $style:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $event as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_event_t>(),
        );
        $event.type_ = YAML_SCALAR_EVENT;
        $event.start_mark = $start_mark;
        $event.end_mark = $end_mark;
        $event.data.scalar.anchor = $anchor;
        $event.data.scalar.tag = $tag;
        $event.data.scalar.value = $value;
        $event.data.scalar.length = $length;
        $event.data.scalar.plain_implicit = $plain_implicit;
        $event.data.scalar.quoted_implicit = $quoted_implicit;
        $event.data.scalar.style = $style;
    }};
}

macro_rules! SEQUENCE_START_EVENT_INIT {
    ($event:expr, $anchor:expr, $tag:expr, $implicit:expr, $style:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $event as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_event_t>(),
        );
        $event.type_ = YAML_SEQUENCE_START_EVENT;
        $event.start_mark = $start_mark;
        $event.end_mark = $end_mark;
        $event.data.sequence_start.anchor = $anchor;
        $event.data.sequence_start.tag = $tag;
        $event.data.sequence_start.implicit = $implicit;
        $event.data.sequence_start.style = $style;
    }};
}

macro_rules! SEQUENCE_END_EVENT_INIT {
    ($event:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $event as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_event_t>(),
        );
        $event.type_ = YAML_SEQUENCE_END_EVENT;
        $event.start_mark = $start_mark;
        $event.end_mark = $end_mark;
    }};
}

macro_rules! MAPPING_START_EVENT_INIT {
    ($event:expr, $anchor:expr, $tag:expr, $implicit:expr, $style:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $event as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_event_t>(),
        );
        $event.type_ = YAML_MAPPING_START_EVENT;
        $event.start_mark = $start_mark;
        $event.end_mark = $end_mark;
        $event.data.mapping_start.anchor = $anchor;
        $event.data.mapping_start.tag = $tag;
        $event.data.mapping_start.implicit = $implicit;
        $event.data.mapping_start.style = $style;
    }};
}

macro_rules! MAPPING_END_EVENT_INIT {
    ($event:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $event as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_event_t>(),
        );
        $event.type_ = YAML_MAPPING_END_EVENT;
        $event.start_mark = $start_mark;
        $event.end_mark = $end_mark;
    }};
}

#[no_mangle]
pub unsafe extern "C" fn yaml_set_max_nest_level(max: c_int) {
    MAX_NESTING_LEVEL = max;
}

/*
 * Get the next event.
 */

#[no_mangle]
pub unsafe extern "C" fn yaml_parser_parse(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
) -> c_int {
    assert!(!parser.is_null());
    assert!(!event.is_null());

    /* Erase the event object. */

    libc::memset(event as *mut libc::c_void, 0, core::mem::size_of::<yaml_event_t>());

    /* No events after the end of the stream or error. */

    if (*parser).stream_end_produced != 0
        || (*parser).error != YAML_NO_ERROR
        || (*parser).state == YAML_PARSE_END_STATE
    {
        return 1;
    }

    /* Generate the next event. */

    yaml_parser_state_machine(parser, event)
}

/*
 * Set parser error.
 */

unsafe fn yaml_parser_set_parser_error(
    parser: *mut yaml_parser_t,
    problem: *const c_char,
    problem_mark: yaml_mark_t,
) -> c_int {
    (*parser).error = YAML_PARSER_ERROR;
    (*parser).problem = problem;
    (*parser).problem_mark = problem_mark;

    0
}

unsafe fn yaml_parser_set_parser_error_context(
    parser: *mut yaml_parser_t,
    context: *const c_char,
    context_mark: yaml_mark_t,
    problem: *const c_char,
    problem_mark: yaml_mark_t,
) -> c_int {
    (*parser).error = YAML_PARSER_ERROR;
    (*parser).context = context;
    (*parser).context_mark = context_mark;
    (*parser).problem = problem;
    (*parser).problem_mark = problem_mark;

    0
}

unsafe fn yaml_maximum_level_reached(
    parser: *mut yaml_parser_t,
    context_mark: yaml_mark_t,
    problem_mark: yaml_mark_t,
) -> c_int {
    yaml_parser_set_parser_error_context(
        parser,
        b"while parsing\0".as_ptr() as *const c_char,
        context_mark,
        b"Maximum nesting level reached, set with yaml_set_max_nest_level())\0".as_ptr()
            as *const c_char,
        problem_mark,
    );
    0
}

/*
 * State dispatcher.
 */

unsafe fn yaml_parser_state_machine(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
) -> c_int {
    match (*parser).state {
        YAML_PARSE_STREAM_START_STATE => yaml_parser_parse_stream_start(parser, event),

        YAML_PARSE_IMPLICIT_DOCUMENT_START_STATE => {
            yaml_parser_parse_document_start(parser, event, 1)
        }

        YAML_PARSE_DOCUMENT_START_STATE => yaml_parser_parse_document_start(parser, event, 0),

        YAML_PARSE_DOCUMENT_CONTENT_STATE => yaml_parser_parse_document_content(parser, event),

        YAML_PARSE_DOCUMENT_END_STATE => yaml_parser_parse_document_end(parser, event),

        YAML_PARSE_BLOCK_NODE_STATE => yaml_parser_parse_node(parser, event, 1, 0),

        YAML_PARSE_BLOCK_NODE_OR_INDENTLESS_SEQUENCE_STATE => {
            yaml_parser_parse_node(parser, event, 1, 1)
        }

        YAML_PARSE_FLOW_NODE_STATE => yaml_parser_parse_node(parser, event, 0, 0),

        YAML_PARSE_BLOCK_SEQUENCE_FIRST_ENTRY_STATE => {
            yaml_parser_parse_block_sequence_entry(parser, event, 1)
        }

        YAML_PARSE_BLOCK_SEQUENCE_ENTRY_STATE => {
            yaml_parser_parse_block_sequence_entry(parser, event, 0)
        }

        YAML_PARSE_INDENTLESS_SEQUENCE_ENTRY_STATE => {
            yaml_parser_parse_indentless_sequence_entry(parser, event)
        }

        YAML_PARSE_BLOCK_MAPPING_FIRST_KEY_STATE => {
            yaml_parser_parse_block_mapping_key(parser, event, 1)
        }

        YAML_PARSE_BLOCK_MAPPING_KEY_STATE => {
            yaml_parser_parse_block_mapping_key(parser, event, 0)
        }

        YAML_PARSE_BLOCK_MAPPING_VALUE_STATE => {
            yaml_parser_parse_block_mapping_value(parser, event)
        }

        YAML_PARSE_FLOW_SEQUENCE_FIRST_ENTRY_STATE => {
            yaml_parser_parse_flow_sequence_entry(parser, event, 1)
        }

        YAML_PARSE_FLOW_SEQUENCE_ENTRY_STATE => {
            yaml_parser_parse_flow_sequence_entry(parser, event, 0)
        }

        YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_KEY_STATE => {
            yaml_parser_parse_flow_sequence_entry_mapping_key(parser, event)
        }

        YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_VALUE_STATE => {
            yaml_parser_parse_flow_sequence_entry_mapping_value(parser, event)
        }

        YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_END_STATE => {
            yaml_parser_parse_flow_sequence_entry_mapping_end(parser, event)
        }

        YAML_PARSE_FLOW_MAPPING_FIRST_KEY_STATE => {
            yaml_parser_parse_flow_mapping_key(parser, event, 1)
        }

        YAML_PARSE_FLOW_MAPPING_KEY_STATE => yaml_parser_parse_flow_mapping_key(parser, event, 0),

        YAML_PARSE_FLOW_MAPPING_VALUE_STATE => {
            yaml_parser_parse_flow_mapping_value(parser, event, 0)
        }

        YAML_PARSE_FLOW_MAPPING_EMPTY_VALUE_STATE => {
            yaml_parser_parse_flow_mapping_value(parser, event, 1)
        }

        _ => {
            assert!(1 != 0); /* Invalid state. */
            0
        }
    }
}

/*
 * Parse the production:
 * stream   ::= STREAM-START implicit_document? explicit_document* STREAM-END
 *              ************
 */

unsafe fn yaml_parser_parse_stream_start(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
) -> c_int {
    let token: *mut yaml_token_t;

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if (*token).type_ != YAML_STREAM_START_TOKEN {
        return yaml_parser_set_parser_error(
            parser,
            b"did not find expected <stream-start>\0".as_ptr() as *const c_char,
            (*token).start_mark,
        );
    }

    (*parser).state = YAML_PARSE_IMPLICIT_DOCUMENT_START_STATE;
    STREAM_START_EVENT_INIT!(
        *event,
        (*token).data.stream_start.encoding,
        (*token).start_mark,
        (*token).start_mark
    );
    skip_token(parser);

    1
}

/*
 * Parse the productions:
 * implicit_document    ::= block_node DOCUMENT-END*
 *                          *
 * explicit_document    ::= DIRECTIVE* DOCUMENT-START block_node? DOCUMENT-END*
 *                          *************************
 */

unsafe fn yaml_parser_parse_document_start(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    implicit: c_int,
) -> c_int {
    let mut token: *mut yaml_token_t;
    let mut version_directive: *mut yaml_version_directive_t = core::ptr::null_mut();
    let mut tag_directives_start: *mut yaml_tag_directive_t = core::ptr::null_mut();
    let mut tag_directives_end: *mut yaml_tag_directive_t = core::ptr::null_mut();

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    /* Parse extra document end indicators. */

    if implicit == 0 {
        while (*token).type_ == YAML_DOCUMENT_END_TOKEN {
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                return 0;
            }
        }
    }

    /* Parse an implicit document. */

    if implicit != 0
        && (*token).type_ != YAML_VERSION_DIRECTIVE_TOKEN
        && (*token).type_ != YAML_TAG_DIRECTIVE_TOKEN
        && (*token).type_ != YAML_DOCUMENT_START_TOKEN
        && (*token).type_ != YAML_STREAM_END_TOKEN
    {
        if yaml_parser_process_directives(
            parser,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            return 0;
        }
        if PUSH!(parser, (*parser).states, YAML_PARSE_DOCUMENT_END_STATE) == 0 {
            return 0;
        }
        (*parser).state = YAML_PARSE_BLOCK_NODE_STATE;
        DOCUMENT_START_EVENT_INIT!(
            *event,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            1i32,
            (*token).start_mark,
            (*token).start_mark
        );
        return 1;
    }

    /* Parse an explicit document. */

    else if (*token).type_ != YAML_STREAM_END_TOKEN {
        let start_mark: yaml_mark_t;
        let mut end_mark: yaml_mark_t;
        start_mark = (*token).start_mark;
        end_mark = (*token).start_mark;

        let ok: c_int = 'error: {
            if yaml_parser_process_directives(
                parser,
                &mut version_directive,
                &mut tag_directives_start,
                &mut tag_directives_end,
            ) == 0
            {
                break 'error 0;
            }
            token = peek_token(parser);
            if token.is_null() {
                break 'error 0;
            }
            if (*token).type_ != YAML_DOCUMENT_START_TOKEN {
                yaml_parser_set_parser_error(
                    parser,
                    b"did not find expected <document start>\0".as_ptr() as *const c_char,
                    (*token).start_mark,
                );
                break 'error 0;
            }
            if PUSH!(parser, (*parser).states, YAML_PARSE_DOCUMENT_END_STATE) == 0 {
                break 'error 0;
            }
            (*parser).state = YAML_PARSE_DOCUMENT_CONTENT_STATE;
            end_mark = (*token).end_mark;
            DOCUMENT_START_EVENT_INIT!(
                *event,
                version_directive,
                tag_directives_start,
                tag_directives_end,
                0i32,
                start_mark,
                end_mark
            );
            skip_token(parser);
            version_directive = core::ptr::null_mut();
            tag_directives_start = core::ptr::null_mut();
            tag_directives_end = core::ptr::null_mut();
            1i32
        };
        if ok == 0 {
            api::yaml_free(version_directive as *mut libc::c_void);
            while tag_directives_start != tag_directives_end {
                tag_directives_end = tag_directives_end.sub(1);
                api::yaml_free((*tag_directives_end).handle as *mut libc::c_void);
                api::yaml_free((*tag_directives_end).prefix as *mut libc::c_void);
            }
            api::yaml_free(tag_directives_start as *mut libc::c_void);
            return 0;
        }
        return 1;
    }

    /* Parse the stream end. */

    else {
        (*parser).state = YAML_PARSE_END_STATE;
        STREAM_END_EVENT_INIT!(*event, (*token).start_mark, (*token).end_mark);
        skip_token(parser);
        return 1;
    }
}

/*
 * Parse the productions:
 * explicit_document    ::= DIRECTIVE* DOCUMENT-START block_node? DOCUMENT-END*
 *                                                    ***********
 */

unsafe fn yaml_parser_parse_document_content(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
) -> c_int {
    let token: *mut yaml_token_t;

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if (*token).type_ == YAML_VERSION_DIRECTIVE_TOKEN
        || (*token).type_ == YAML_TAG_DIRECTIVE_TOKEN
        || (*token).type_ == YAML_DOCUMENT_START_TOKEN
        || (*token).type_ == YAML_DOCUMENT_END_TOKEN
        || (*token).type_ == YAML_STREAM_END_TOKEN
    {
        (*parser).state = POP!(parser, (*parser).states);
        return yaml_parser_process_empty_scalar(parser, event, (*token).start_mark);
    } else {
        return yaml_parser_parse_node(parser, event, 1, 0);
    }
}

/*
 * Parse the productions:
 * implicit_document    ::= block_node DOCUMENT-END*
 *                                     *************
 * explicit_document    ::= DIRECTIVE* DOCUMENT-START block_node? DOCUMENT-END*
 *                                                                *************
 */

unsafe fn yaml_parser_parse_document_end(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
) -> c_int {
    let token: *mut yaml_token_t;
    let start_mark: yaml_mark_t;
    let mut end_mark: yaml_mark_t;
    let mut implicit: c_int = 1;

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    start_mark = (*token).start_mark;
    end_mark = (*token).start_mark;

    if (*token).type_ == YAML_DOCUMENT_END_TOKEN {
        end_mark = (*token).end_mark;
        skip_token(parser);
        implicit = 0;
    }

    while !STACK_EMPTY!(parser, (*parser).tag_directives) {
        let tag_directive: yaml_tag_directive_t = POP!(parser, (*parser).tag_directives);
        api::yaml_free(tag_directive.handle as *mut libc::c_void);
        api::yaml_free(tag_directive.prefix as *mut libc::c_void);
    }

    (*parser).state = YAML_PARSE_DOCUMENT_START_STATE;
    DOCUMENT_END_EVENT_INIT!(*event, implicit, start_mark, end_mark);

    1
}

/*
 * Parse the productions:
 * block_node_or_indentless_sequence    ::=
 *                          ALIAS
 *                          *****
 *                          | properties (block_content | indentless_block_sequence)?
 *                            **********  *
 *                          | block_content | indentless_block_sequence
 *                            *
 * block_node           ::= ALIAS
 *                          *****
 *                          | properties block_content?
 *                            ********** *
 *                          | block_content
 *                            *
 * flow_node            ::= ALIAS
 *                          *****
 *                          | properties flow_content?
 *                            ********** *
 *                          | flow_content
 *                            *
 * properties           ::= TAG ANCHOR? | ANCHOR TAG?
 *                          *************************
 * block_content        ::= block_collection | flow_collection | SCALAR
 *                                                               ******
 * flow_content         ::= flow_collection | SCALAR
 *                                            ******
 */

unsafe fn yaml_parser_parse_node(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    block: c_int,
    indentless_sequence: c_int,
) -> c_int {
    let mut token: *mut yaml_token_t;
    let mut anchor: *mut yaml_char_t = core::ptr::null_mut();
    let mut tag_handle: *mut yaml_char_t = core::ptr::null_mut();
    let mut tag_suffix: *mut yaml_char_t = core::ptr::null_mut();
    let mut tag: *mut yaml_char_t = core::ptr::null_mut();
    let mut start_mark: yaml_mark_t = yaml_mark_t { index: 0, line: 0, column: 0 };
    let mut end_mark: yaml_mark_t = yaml_mark_t { index: 0, line: 0, column: 0 };
    let mut tag_mark: yaml_mark_t = yaml_mark_t {
        index: 0,
        line: 0,
        column: 0,
    };
    let mut implicit: c_int = 0;

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if (*token).type_ == YAML_ALIAS_TOKEN {
        (*parser).state = POP!(parser, (*parser).states);
        ALIAS_EVENT_INIT!(*event, (*token).data.alias.value, (*token).start_mark, (*token).end_mark);
        skip_token(parser);
        return 1;
    } else {
        start_mark = (*token).start_mark;
        end_mark = (*token).start_mark;

        let ok: c_int = 'error: {
            if (*token).type_ == YAML_ANCHOR_TOKEN {
                anchor = (*token).data.anchor.value;
                start_mark = (*token).start_mark;
                end_mark = (*token).end_mark;
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    break 'error 0;
                }
                if (*token).type_ == YAML_TAG_TOKEN {
                    tag_handle = (*token).data.tag.handle;
                    tag_suffix = (*token).data.tag.suffix;
                    tag_mark = (*token).start_mark;
                    end_mark = (*token).end_mark;
                    skip_token(parser);
                    token = peek_token(parser);
                    if token.is_null() {
                        break 'error 0;
                    }
                }
            } else if (*token).type_ == YAML_TAG_TOKEN {
                tag_handle = (*token).data.tag.handle;
                tag_suffix = (*token).data.tag.suffix;
                start_mark = (*token).start_mark;
                tag_mark = (*token).start_mark;
                end_mark = (*token).end_mark;
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    break 'error 0;
                }
                if (*token).type_ == YAML_ANCHOR_TOKEN {
                    anchor = (*token).data.anchor.value;
                    end_mark = (*token).end_mark;
                    skip_token(parser);
                    token = peek_token(parser);
                    if token.is_null() {
                        break 'error 0;
                    }
                }
            }

            if !tag_handle.is_null() {
                if *tag_handle == 0 {
                    tag = tag_suffix;
                    api::yaml_free(tag_handle as *mut libc::c_void);
                    tag_handle = core::ptr::null_mut();
                    tag_suffix = core::ptr::null_mut();
                } else {
                    let mut tag_directive: *mut yaml_tag_directive_t =
                        (*parser).tag_directives.start;
                    while tag_directive != (*parser).tag_directives.top {
                        if libc::strcmp(
                            (*tag_directive).handle as *const libc::c_char,
                            tag_handle as *const libc::c_char,
                        ) == 0
                        {
                            let prefix_len =
                                libc::strlen((*tag_directive).prefix as *const libc::c_char);
                            let suffix_len = libc::strlen(tag_suffix as *const libc::c_char);
                            tag = YAML_MALLOC(prefix_len + suffix_len + 1);
                            if tag.is_null() {
                                (*parser).error = YAML_MEMORY_ERROR;
                                break 'error 0;
                            }
                            libc::memcpy(
                                tag as *mut libc::c_void,
                                (*tag_directive).prefix as *const libc::c_void,
                                prefix_len,
                            );
                            libc::memcpy(
                                tag.add(prefix_len) as *mut libc::c_void,
                                tag_suffix as *const libc::c_void,
                                suffix_len,
                            );
                            *tag.add(prefix_len + suffix_len) = b'\0';
                            api::yaml_free(tag_handle as *mut libc::c_void);
                            api::yaml_free(tag_suffix as *mut libc::c_void);
                            tag_handle = core::ptr::null_mut();
                            tag_suffix = core::ptr::null_mut();
                            break;
                        }
                        tag_directive = tag_directive.add(1);
                    }
                    if tag.is_null() {
                        yaml_parser_set_parser_error_context(
                            parser,
                            b"while parsing a node\0".as_ptr() as *const c_char,
                            start_mark,
                            b"found undefined tag handle\0".as_ptr() as *const c_char,
                            tag_mark,
                        );
                        break 'error 0;
                    }
                }
            }

            implicit = (tag.is_null() || *tag == 0) as c_int;
            if indentless_sequence != 0 && (*token).type_ == YAML_BLOCK_ENTRY_TOKEN {
                end_mark = (*token).end_mark;
                (*parser).state = YAML_PARSE_INDENTLESS_SEQUENCE_ENTRY_STATE;
                SEQUENCE_START_EVENT_INIT!(
                    *event,
                    anchor,
                    tag,
                    implicit,
                    YAML_BLOCK_SEQUENCE_STYLE,
                    start_mark,
                    end_mark
                );
                return 1;
            } else {
                if (*token).type_ == YAML_SCALAR_TOKEN {
                    let plain_implicit: c_int;
                    let quoted_implicit: c_int;
                    end_mark = (*token).end_mark;
                    if ((*token).data.scalar.style == YAML_PLAIN_SCALAR_STYLE && tag.is_null())
                        || (!tag.is_null()
                            && libc::strcmp(
                                tag as *const libc::c_char,
                                b"!\0".as_ptr() as *const libc::c_char,
                            ) == 0)
                    {
                        plain_implicit = 1;
                        quoted_implicit = 0;
                    } else if tag.is_null() {
                        quoted_implicit = 1;
                        plain_implicit = 0;
                    } else {
                        plain_implicit = 0;
                        quoted_implicit = 0;
                    }
                    (*parser).state = POP!(parser, (*parser).states);
                    SCALAR_EVENT_INIT!(
                        *event,
                        anchor,
                        tag,
                        (*token).data.scalar.value,
                        (*token).data.scalar.length,
                        plain_implicit,
                        quoted_implicit,
                        (*token).data.scalar.style,
                        start_mark,
                        end_mark
                    );
                    skip_token(parser);
                    return 1;
                } else if (*token).type_ == YAML_FLOW_SEQUENCE_START_TOKEN {
                    if STACK_LIMIT!(
                        parser,
                        (*parser).indents,
                        MAX_NESTING_LEVEL - (*parser).flow_level
                    ) == 0
                    {
                        yaml_maximum_level_reached(parser, start_mark, (*token).start_mark);
                        break 'error 0;
                    }
                    end_mark = (*token).end_mark;
                    (*parser).state = YAML_PARSE_FLOW_SEQUENCE_FIRST_ENTRY_STATE;
                    SEQUENCE_START_EVENT_INIT!(
                        *event,
                        anchor,
                        tag,
                        implicit,
                        YAML_FLOW_SEQUENCE_STYLE,
                        start_mark,
                        end_mark
                    );
                    return 1;
                } else if (*token).type_ == YAML_FLOW_MAPPING_START_TOKEN {
                    if STACK_LIMIT!(
                        parser,
                        (*parser).indents,
                        MAX_NESTING_LEVEL - (*parser).flow_level
                    ) == 0
                    {
                        yaml_maximum_level_reached(parser, start_mark, (*token).start_mark);
                        break 'error 0;
                    }
                    end_mark = (*token).end_mark;
                    (*parser).state = YAML_PARSE_FLOW_MAPPING_FIRST_KEY_STATE;
                    MAPPING_START_EVENT_INIT!(
                        *event,
                        anchor,
                        tag,
                        implicit,
                        YAML_FLOW_MAPPING_STYLE,
                        start_mark,
                        end_mark
                    );
                    return 1;
                } else if block != 0 && (*token).type_ == YAML_BLOCK_SEQUENCE_START_TOKEN {
                    if STACK_LIMIT!(
                        parser,
                        (*parser).indents,
                        MAX_NESTING_LEVEL - (*parser).flow_level
                    ) == 0
                    {
                        yaml_maximum_level_reached(parser, start_mark, (*token).start_mark);
                        break 'error 0;
                    }
                    end_mark = (*token).end_mark;
                    (*parser).state = YAML_PARSE_BLOCK_SEQUENCE_FIRST_ENTRY_STATE;
                    SEQUENCE_START_EVENT_INIT!(
                        *event,
                        anchor,
                        tag,
                        implicit,
                        YAML_BLOCK_SEQUENCE_STYLE,
                        start_mark,
                        end_mark
                    );
                    return 1;
                } else if block != 0 && (*token).type_ == YAML_BLOCK_MAPPING_START_TOKEN {
                    if STACK_LIMIT!(
                        parser,
                        (*parser).indents,
                        MAX_NESTING_LEVEL - (*parser).flow_level
                    ) == 0
                    {
                        yaml_maximum_level_reached(parser, start_mark, (*token).start_mark);
                        break 'error 0;
                    }
                    end_mark = (*token).end_mark;
                    (*parser).state = YAML_PARSE_BLOCK_MAPPING_FIRST_KEY_STATE;
                    MAPPING_START_EVENT_INIT!(
                        *event,
                        anchor,
                        tag,
                        implicit,
                        YAML_BLOCK_MAPPING_STYLE,
                        start_mark,
                        end_mark
                    );
                    return 1;
                } else if !anchor.is_null() || !tag.is_null() {
                    let value: *mut yaml_char_t = YAML_MALLOC(1);
                    if value.is_null() {
                        (*parser).error = YAML_MEMORY_ERROR;
                        break 'error 0;
                    }
                    *value = b'\0';
                    (*parser).state = POP!(parser, (*parser).states);
                    SCALAR_EVENT_INIT!(
                        *event,
                        anchor,
                        tag,
                        value,
                        0usize,
                        implicit,
                        0i32,
                        YAML_PLAIN_SCALAR_STYLE,
                        start_mark,
                        end_mark
                    );
                    return 1;
                } else {
                    yaml_parser_set_parser_error_context(
                        parser,
                        if block != 0 {
                            b"while parsing a block node\0".as_ptr() as *const c_char
                        } else {
                            b"while parsing a flow node\0".as_ptr() as *const c_char
                        },
                        start_mark,
                        b"did not find expected node content\0".as_ptr() as *const c_char,
                        (*token).start_mark,
                    );
                    break 'error 0;
                }
            }
        };
        if ok == 0 {
            api::yaml_free(anchor as *mut libc::c_void);
            api::yaml_free(tag_handle as *mut libc::c_void);
            api::yaml_free(tag_suffix as *mut libc::c_void);
            api::yaml_free(tag as *mut libc::c_void);
            return 0;
        }
        return ok;
    }
}

/*
 * Parse the productions:
 * block_sequence ::= BLOCK-SEQUENCE-START (BLOCK-ENTRY block_node?)* BLOCK-END
 *                    ********************  *********** *             *********
 */

unsafe fn yaml_parser_parse_block_sequence_entry(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    first: c_int,
) -> c_int {
    let mut token: *mut yaml_token_t;

    if first != 0 {
        token = peek_token(parser);
        if PUSH!(parser, (*parser).marks, (*token).start_mark) == 0 {
            return 0;
        }
        skip_token(parser);
    }

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if (*token).type_ == YAML_BLOCK_ENTRY_TOKEN {
        let mark: yaml_mark_t = (*token).end_mark;
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return 0;
        }
        if (*token).type_ != YAML_BLOCK_ENTRY_TOKEN && (*token).type_ != YAML_BLOCK_END_TOKEN {
            if PUSH!(parser, (*parser).states, YAML_PARSE_BLOCK_SEQUENCE_ENTRY_STATE) == 0 {
                return 0;
            }
            return yaml_parser_parse_node(parser, event, 1, 0);
        } else {
            (*parser).state = YAML_PARSE_BLOCK_SEQUENCE_ENTRY_STATE;
            return yaml_parser_process_empty_scalar(parser, event, mark);
        }
    } else if (*token).type_ == YAML_BLOCK_END_TOKEN {
        (*parser).state = POP!(parser, (*parser).states);
        let _ = POP!(parser, (*parser).marks);
        SEQUENCE_END_EVENT_INIT!(*event, (*token).start_mark, (*token).end_mark);
        skip_token(parser);
        return 1;
    } else {
        return yaml_parser_set_parser_error_context(
            parser,
            b"while parsing a block collection\0".as_ptr() as *const c_char,
            POP!(parser, (*parser).marks),
            b"did not find expected '-' indicator\0".as_ptr() as *const c_char,
            (*token).start_mark,
        );
    }
}

/*
 * Parse the productions:
 * indentless_sequence  ::= (BLOCK-ENTRY block_node?)+
 *                           *********** *
 */

unsafe fn yaml_parser_parse_indentless_sequence_entry(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
) -> c_int {
    let mut token: *mut yaml_token_t;

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if (*token).type_ == YAML_BLOCK_ENTRY_TOKEN {
        let mark: yaml_mark_t = (*token).end_mark;
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return 0;
        }
        if (*token).type_ != YAML_BLOCK_ENTRY_TOKEN
            && (*token).type_ != YAML_KEY_TOKEN
            && (*token).type_ != YAML_VALUE_TOKEN
            && (*token).type_ != YAML_BLOCK_END_TOKEN
        {
            if PUSH!(
                parser,
                (*parser).states,
                YAML_PARSE_INDENTLESS_SEQUENCE_ENTRY_STATE
            ) == 0
            {
                return 0;
            }
            return yaml_parser_parse_node(parser, event, 1, 0);
        } else {
            (*parser).state = YAML_PARSE_INDENTLESS_SEQUENCE_ENTRY_STATE;
            return yaml_parser_process_empty_scalar(parser, event, mark);
        }
    } else {
        (*parser).state = POP!(parser, (*parser).states);
        SEQUENCE_END_EVENT_INIT!(*event, (*token).start_mark, (*token).start_mark);
        return 1;
    }
}

/*
 * Parse the productions:
 * block_mapping        ::= BLOCK-MAPPING_START
 *                          *******************
 *                          ((KEY block_node_or_indentless_sequence?)?
 *                            *** *
 *                          (VALUE block_node_or_indentless_sequence?)?)*
 *
 *                          BLOCK-END
 *                          *********
 */

unsafe fn yaml_parser_parse_block_mapping_key(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    first: c_int,
) -> c_int {
    let mut token: *mut yaml_token_t;

    if first != 0 {
        token = peek_token(parser);
        if PUSH!(parser, (*parser).marks, (*token).start_mark) == 0 {
            return 0;
        }
        skip_token(parser);
    }

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if (*token).type_ == YAML_KEY_TOKEN {
        let mark: yaml_mark_t = (*token).end_mark;
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return 0;
        }
        if (*token).type_ != YAML_KEY_TOKEN
            && (*token).type_ != YAML_VALUE_TOKEN
            && (*token).type_ != YAML_BLOCK_END_TOKEN
        {
            if PUSH!(parser, (*parser).states, YAML_PARSE_BLOCK_MAPPING_VALUE_STATE) == 0 {
                return 0;
            }
            return yaml_parser_parse_node(parser, event, 1, 1);
        } else {
            (*parser).state = YAML_PARSE_BLOCK_MAPPING_VALUE_STATE;
            return yaml_parser_process_empty_scalar(parser, event, mark);
        }
    } else if (*token).type_ == YAML_BLOCK_END_TOKEN {
        (*parser).state = POP!(parser, (*parser).states);
        let _ = POP!(parser, (*parser).marks);
        MAPPING_END_EVENT_INIT!(*event, (*token).start_mark, (*token).end_mark);
        skip_token(parser);
        return 1;
    } else {
        return yaml_parser_set_parser_error_context(
            parser,
            b"while parsing a block mapping\0".as_ptr() as *const c_char,
            POP!(parser, (*parser).marks),
            b"did not find expected key\0".as_ptr() as *const c_char,
            (*token).start_mark,
        );
    }
}

/*
 * Parse the productions:
 * block_mapping        ::= BLOCK-MAPPING_START
 *
 *                          ((KEY block_node_or_indentless_sequence?)?
 *
 *                          (VALUE block_node_or_indentless_sequence?)?)*
 *                           ***** *
 *                          BLOCK-END
 *
 */

unsafe fn yaml_parser_parse_block_mapping_value(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
) -> c_int {
    let mut token: *mut yaml_token_t;

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if (*token).type_ == YAML_VALUE_TOKEN {
        let mark: yaml_mark_t = (*token).end_mark;
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return 0;
        }
        if (*token).type_ != YAML_KEY_TOKEN
            && (*token).type_ != YAML_VALUE_TOKEN
            && (*token).type_ != YAML_BLOCK_END_TOKEN
        {
            if PUSH!(parser, (*parser).states, YAML_PARSE_BLOCK_MAPPING_KEY_STATE) == 0 {
                return 0;
            }
            return yaml_parser_parse_node(parser, event, 1, 1);
        } else {
            (*parser).state = YAML_PARSE_BLOCK_MAPPING_KEY_STATE;
            return yaml_parser_process_empty_scalar(parser, event, mark);
        }
    } else {
        (*parser).state = YAML_PARSE_BLOCK_MAPPING_KEY_STATE;
        return yaml_parser_process_empty_scalar(parser, event, (*token).start_mark);
    }
}

/*
 * Parse the productions:
 * flow_sequence        ::= FLOW-SEQUENCE-START
 *                          *******************
 *                          (flow_sequence_entry FLOW-ENTRY)*
 *                           *                   **********
 *                          flow_sequence_entry?
 *                          *
 *                          FLOW-SEQUENCE-END
 *                          *****************
 * flow_sequence_entry  ::= flow_node | KEY flow_node? (VALUE flow_node?)?
 *                          *
 */

unsafe fn yaml_parser_parse_flow_sequence_entry(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    first: c_int,
) -> c_int {
    let mut token: *mut yaml_token_t;

    if first != 0 {
        token = peek_token(parser);
        if PUSH!(parser, (*parser).marks, (*token).start_mark) == 0 {
            return 0;
        }
        skip_token(parser);
    }

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if (*token).type_ != YAML_FLOW_SEQUENCE_END_TOKEN {
        if first == 0 {
            if (*token).type_ == YAML_FLOW_ENTRY_TOKEN {
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    return 0;
                }
            } else {
                return yaml_parser_set_parser_error_context(
                    parser,
                    b"while parsing a flow sequence\0".as_ptr() as *const c_char,
                    POP!(parser, (*parser).marks),
                    b"did not find expected ',' or ']'\0".as_ptr() as *const c_char,
                    (*token).start_mark,
                );
            }
        }

        if (*token).type_ == YAML_KEY_TOKEN {
            (*parser).state = YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_KEY_STATE;
            MAPPING_START_EVENT_INIT!(
                *event,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                1i32,
                YAML_FLOW_MAPPING_STYLE,
                (*token).start_mark,
                (*token).end_mark
            );
            skip_token(parser);
            return 1;
        } else if (*token).type_ != YAML_FLOW_SEQUENCE_END_TOKEN {
            if PUSH!(parser, (*parser).states, YAML_PARSE_FLOW_SEQUENCE_ENTRY_STATE) == 0 {
                return 0;
            }
            return yaml_parser_parse_node(parser, event, 0, 0);
        }
    }

    (*parser).state = POP!(parser, (*parser).states);
    let _ = POP!(parser, (*parser).marks);
    SEQUENCE_END_EVENT_INIT!(*event, (*token).start_mark, (*token).end_mark);
    skip_token(parser);
    1
}

/*
 * Parse the productions:
 * flow_sequence_entry  ::= flow_node | KEY flow_node? (VALUE flow_node?)?
 *                                      *** *
 */

unsafe fn yaml_parser_parse_flow_sequence_entry_mapping_key(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
) -> c_int {
    let mut token: *mut yaml_token_t;

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if (*token).type_ != YAML_VALUE_TOKEN
        && (*token).type_ != YAML_FLOW_ENTRY_TOKEN
        && (*token).type_ != YAML_FLOW_SEQUENCE_END_TOKEN
    {
        if PUSH!(
            parser,
            (*parser).states,
            YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_VALUE_STATE
        ) == 0
        {
            return 0;
        }
        return yaml_parser_parse_node(parser, event, 0, 0);
    } else if (*token).type_ == YAML_FLOW_SEQUENCE_END_TOKEN {
        let mark: yaml_mark_t = (*token).start_mark;
        (*parser).state = YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_VALUE_STATE;
        return yaml_parser_process_empty_scalar(parser, event, mark);
    } else {
        let mark: yaml_mark_t = (*token).end_mark;
        skip_token(parser);
        (*parser).state = YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_VALUE_STATE;
        return yaml_parser_process_empty_scalar(parser, event, mark);
    }
}

/*
 * Parse the productions:
 * flow_sequence_entry  ::= flow_node | KEY flow_node? (VALUE flow_node?)?
 *                                                      ***** *
 */

unsafe fn yaml_parser_parse_flow_sequence_entry_mapping_value(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
) -> c_int {
    let mut token: *mut yaml_token_t;

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if (*token).type_ == YAML_VALUE_TOKEN {
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return 0;
        }
        if (*token).type_ != YAML_FLOW_ENTRY_TOKEN
            && (*token).type_ != YAML_FLOW_SEQUENCE_END_TOKEN
        {
            if PUSH!(
                parser,
                (*parser).states,
                YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_END_STATE
            ) == 0
            {
                return 0;
            }
            return yaml_parser_parse_node(parser, event, 0, 0);
        }
    }
    (*parser).state = YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_END_STATE;
    yaml_parser_process_empty_scalar(parser, event, (*token).start_mark)
}

/*
 * Parse the productions:
 * flow_sequence_entry  ::= flow_node | KEY flow_node? (VALUE flow_node?)?
 *                                                                      *
 */

unsafe fn yaml_parser_parse_flow_sequence_entry_mapping_end(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
) -> c_int {
    let token: *mut yaml_token_t;

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    (*parser).state = YAML_PARSE_FLOW_SEQUENCE_ENTRY_STATE;

    MAPPING_END_EVENT_INIT!(*event, (*token).start_mark, (*token).start_mark);
    1
}

/*
 * Parse the productions:
 * flow_mapping         ::= FLOW-MAPPING-START
 *                          ******************
 *                          (flow_mapping_entry FLOW-ENTRY)*
 *                           *                  **********
 *                          flow_mapping_entry?
 *                          ******************
 *                          FLOW-MAPPING-END
 *                          ****************
 * flow_mapping_entry   ::= flow_node | KEY flow_node? (VALUE flow_node?)?
 *                          *           *** *
 */

unsafe fn yaml_parser_parse_flow_mapping_key(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    first: c_int,
) -> c_int {
    let mut token: *mut yaml_token_t;

    if first != 0 {
        token = peek_token(parser);
        if PUSH!(parser, (*parser).marks, (*token).start_mark) == 0 {
            return 0;
        }
        skip_token(parser);
    }

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if (*token).type_ != YAML_FLOW_MAPPING_END_TOKEN {
        if first == 0 {
            if (*token).type_ == YAML_FLOW_ENTRY_TOKEN {
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    return 0;
                }
            } else {
                return yaml_parser_set_parser_error_context(
                    parser,
                    b"while parsing a flow mapping\0".as_ptr() as *const c_char,
                    POP!(parser, (*parser).marks),
                    b"did not find expected ',' or '}'\0".as_ptr() as *const c_char,
                    (*token).start_mark,
                );
            }
        }

        if (*token).type_ == YAML_KEY_TOKEN {
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                return 0;
            }
            if (*token).type_ != YAML_VALUE_TOKEN
                && (*token).type_ != YAML_FLOW_ENTRY_TOKEN
                && (*token).type_ != YAML_FLOW_MAPPING_END_TOKEN
            {
                if PUSH!(parser, (*parser).states, YAML_PARSE_FLOW_MAPPING_VALUE_STATE) == 0 {
                    return 0;
                }
                return yaml_parser_parse_node(parser, event, 0, 0);
            } else {
                (*parser).state = YAML_PARSE_FLOW_MAPPING_VALUE_STATE;
                return yaml_parser_process_empty_scalar(parser, event, (*token).start_mark);
            }
        } else if (*token).type_ != YAML_FLOW_MAPPING_END_TOKEN {
            if PUSH!(parser, (*parser).states, YAML_PARSE_FLOW_MAPPING_EMPTY_VALUE_STATE) == 0 {
                return 0;
            }
            return yaml_parser_parse_node(parser, event, 0, 0);
        }
    }

    (*parser).state = POP!(parser, (*parser).states);
    let _ = POP!(parser, (*parser).marks);
    MAPPING_END_EVENT_INIT!(*event, (*token).start_mark, (*token).end_mark);
    skip_token(parser);
    1
}

/*
 * Parse the productions:
 * flow_mapping_entry   ::= flow_node | KEY flow_node? (VALUE flow_node?)?
 *                                   *                  ***** *
 */

unsafe fn yaml_parser_parse_flow_mapping_value(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    empty: c_int,
) -> c_int {
    let mut token: *mut yaml_token_t;

    token = peek_token(parser);
    if token.is_null() {
        return 0;
    }

    if empty != 0 {
        (*parser).state = YAML_PARSE_FLOW_MAPPING_KEY_STATE;
        return yaml_parser_process_empty_scalar(parser, event, (*token).start_mark);
    }

    if (*token).type_ == YAML_VALUE_TOKEN {
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return 0;
        }
        if (*token).type_ != YAML_FLOW_ENTRY_TOKEN && (*token).type_ != YAML_FLOW_MAPPING_END_TOKEN
        {
            if PUSH!(parser, (*parser).states, YAML_PARSE_FLOW_MAPPING_KEY_STATE) == 0 {
                return 0;
            }
            return yaml_parser_parse_node(parser, event, 0, 0);
        }
    }

    (*parser).state = YAML_PARSE_FLOW_MAPPING_KEY_STATE;
    yaml_parser_process_empty_scalar(parser, event, (*token).start_mark)
}

/*
 * Generate an empty scalar event.
 */

unsafe fn yaml_parser_process_empty_scalar(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    mark: yaml_mark_t,
) -> c_int {
    let value: *mut yaml_char_t;

    value = YAML_MALLOC(1);
    if value.is_null() {
        (*parser).error = YAML_MEMORY_ERROR;
        return 0;
    }
    *value = b'\0';

    SCALAR_EVENT_INIT!(
        *event,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        value,
        0usize,
        1i32,
        0i32,
        YAML_PLAIN_SCALAR_STYLE,
        mark,
        mark
    );

    1
}

/*
 * Parse directives.
 */

unsafe fn yaml_parser_process_directives(
    parser: *mut yaml_parser_t,
    version_directive_ref: *mut *mut yaml_version_directive_t,
    tag_directives_start_ref: *mut *mut yaml_tag_directive_t,
    tag_directives_end_ref: *mut *mut yaml_tag_directive_t,
) -> c_int {
    // default_tag_directives as static strings
    let default_handle_1: *mut yaml_char_t = b"!\0".as_ptr() as *mut yaml_char_t;
    let default_prefix_1: *mut yaml_char_t = b"!\0".as_ptr() as *mut yaml_char_t;
    let default_handle_2: *mut yaml_char_t = b"!!\0".as_ptr() as *mut yaml_char_t;
    let default_prefix_2: *mut yaml_char_t =
        b"tag:yaml.org,2002:\0".as_ptr() as *mut yaml_char_t;

    let default_tag_directives: [yaml_tag_directive_t; 3] = [
        yaml_tag_directive_t {
            handle: default_handle_1,
            prefix: default_prefix_1,
        },
        yaml_tag_directive_t {
            handle: default_handle_2,
            prefix: default_prefix_2,
        },
        yaml_tag_directive_t {
            handle: core::ptr::null_mut(),
            prefix: core::ptr::null_mut(),
        },
    ];

    let mut version_directive: *mut yaml_version_directive_t = core::ptr::null_mut();
    // tag_directives is a local stack
    let mut tag_directives = yaml_tag_directive_stack_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        top: core::ptr::null_mut(),
    };
    let mut token: *mut yaml_token_t = core::ptr::null_mut();

    let ok: c_int = 'error: {
        if STACK_INIT!(parser, tag_directives, yaml_tag_directive_t) == 0 {
            break 'error 0;
        }

        token = peek_token(parser);
        if token.is_null() {
            break 'error 0;
        }

        while (*token).type_ == YAML_VERSION_DIRECTIVE_TOKEN
            || (*token).type_ == YAML_TAG_DIRECTIVE_TOKEN
        {
            if (*token).type_ == YAML_VERSION_DIRECTIVE_TOKEN {
                if !version_directive.is_null() {
                    yaml_parser_set_parser_error(
                        parser,
                        b"found duplicate %YAML directive\0".as_ptr() as *const c_char,
                        (*token).start_mark,
                    );
                    break 'error 0;
                }
                if (*token).data.version_directive.major != 1
                    || ((*token).data.version_directive.minor != 1
                        && (*token).data.version_directive.minor != 2)
                {
                    yaml_parser_set_parser_error(
                        parser,
                        b"found incompatible YAML document\0".as_ptr() as *const c_char,
                        (*token).start_mark,
                    );
                    break 'error 0;
                }
                version_directive = YAML_MALLOC_STATIC::<yaml_version_directive_t>();
                if version_directive.is_null() {
                    (*parser).error = YAML_MEMORY_ERROR;
                    break 'error 0;
                }
                (*version_directive).major = (*token).data.version_directive.major;
                (*version_directive).minor = (*token).data.version_directive.minor;
            } else if (*token).type_ == YAML_TAG_DIRECTIVE_TOKEN {
                let value = yaml_tag_directive_t {
                    handle: (*token).data.tag_directive.handle,
                    prefix: (*token).data.tag_directive.prefix,
                };

                if yaml_parser_append_tag_directive(parser, value, 0, (*token).start_mark) == 0 {
                    break 'error 0;
                }
                if PUSH!(parser, tag_directives, value) == 0 {
                    break 'error 0;
                }
            }

            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                break 'error 0;
            }
        }

        let mut idx: usize = 0;
        loop {
            let default_tag_directive = &default_tag_directives[idx];
            if default_tag_directive.handle.is_null() {
                break;
            }
            if yaml_parser_append_tag_directive(
                parser,
                *default_tag_directive,
                1,
                (*token).start_mark,
            ) == 0
            {
                break 'error 0;
            }
            idx += 1;
        }

        if !version_directive_ref.is_null() {
            *version_directive_ref = version_directive;
        }
        if !tag_directives_start_ref.is_null() {
            if STACK_EMPTY!(parser, tag_directives) {
                *tag_directives_start_ref = core::ptr::null_mut();
                *tag_directives_end_ref = core::ptr::null_mut();
                STACK_DEL!(parser, tag_directives);
            } else {
                *tag_directives_start_ref = tag_directives.start;
                *tag_directives_end_ref = tag_directives.top;
            }
        } else {
            STACK_DEL!(parser, tag_directives);
        }

        if version_directive_ref.is_null() {
            api::yaml_free(version_directive as *mut libc::c_void);
        }
        1i32
    };

    if ok == 0 {
        api::yaml_free(version_directive as *mut libc::c_void);
        while !STACK_EMPTY!(parser, tag_directives) {
            let tag_directive: yaml_tag_directive_t = POP!(parser, tag_directives);
            api::yaml_free(tag_directive.handle as *mut libc::c_void);
            api::yaml_free(tag_directive.prefix as *mut libc::c_void);
        }
        STACK_DEL!(parser, tag_directives);
        return 0;
    }
    ok
}

/*
 * Append a tag directive to the directives stack.
 */

unsafe fn yaml_parser_append_tag_directive(
    parser: *mut yaml_parser_t,
    value: yaml_tag_directive_t,
    allow_duplicates: c_int,
    mark: yaml_mark_t,
) -> c_int {
    let mut tag_directive: *mut yaml_tag_directive_t = (*parser).tag_directives.start;
    let mut copy = yaml_tag_directive_t {
        handle: core::ptr::null_mut(),
        prefix: core::ptr::null_mut(),
    };

    while tag_directive != (*parser).tag_directives.top {
        if libc::strcmp(
            value.handle as *const libc::c_char,
            (*tag_directive).handle as *const libc::c_char,
        ) == 0
        {
            if allow_duplicates != 0 {
                return 1;
            }
            return yaml_parser_set_parser_error(
                parser,
                b"found duplicate %TAG directive\0".as_ptr() as *const c_char,
                mark,
            );
        }
        tag_directive = tag_directive.add(1);
    }

    copy.handle = api::yaml_strdup(value.handle);
    copy.prefix = api::yaml_strdup(value.prefix);
    if copy.handle.is_null() || copy.prefix.is_null() {
        (*parser).error = YAML_MEMORY_ERROR;
        // goto error
        api::yaml_free(copy.handle as *mut libc::c_void);
        api::yaml_free(copy.prefix as *mut libc::c_void);
        return 0;
    }

    if PUSH!(parser, (*parser).tag_directives, copy) == 0 {
        api::yaml_free(copy.handle as *mut libc::c_void);
        api::yaml_free(copy.prefix as *mut libc::c_void);
        return 0;
    }

    1
}
