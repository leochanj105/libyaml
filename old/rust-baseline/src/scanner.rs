use crate::*;
use libc::{c_char, c_int, c_long, size_t, ptrdiff_t};

macro_rules! STREAM_START_TOKEN_INIT {
    ($token:expr, $encoding:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $token as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_token_t>(),
        );
        $token.type_ = YAML_STREAM_START_TOKEN;
        $token.start_mark = $start_mark;
        $token.end_mark = $end_mark;
        $token.data.stream_start.encoding = $encoding;
    }};
}

macro_rules! STREAM_END_TOKEN_INIT {
    ($token:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $token as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_token_t>(),
        );
        $token.type_ = YAML_STREAM_END_TOKEN;
        $token.start_mark = $start_mark;
        $token.end_mark = $end_mark;
    }};
}

macro_rules! VERSION_DIRECTIVE_TOKEN_INIT {
    ($token:expr, $major:expr, $minor:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $token as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_token_t>(),
        );
        $token.type_ = YAML_VERSION_DIRECTIVE_TOKEN;
        $token.start_mark = $start_mark;
        $token.end_mark = $end_mark;
        $token.data.version_directive.major = $major;
        $token.data.version_directive.minor = $minor;
    }};
}

macro_rules! TAG_DIRECTIVE_TOKEN_INIT {
    ($token:expr, $handle:expr, $prefix:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $token as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_token_t>(),
        );
        $token.type_ = YAML_TAG_DIRECTIVE_TOKEN;
        $token.start_mark = $start_mark;
        $token.end_mark = $end_mark;
        $token.data.tag_directive.handle = $handle;
        $token.data.tag_directive.prefix = $prefix;
    }};
}

macro_rules! ALIAS_TOKEN_INIT {
    ($token:expr, $value:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $token as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_token_t>(),
        );
        $token.type_ = YAML_ALIAS_TOKEN;
        $token.start_mark = $start_mark;
        $token.end_mark = $end_mark;
        $token.data.alias.value = $value;
    }};
}

macro_rules! ANCHOR_TOKEN_INIT {
    ($token:expr, $value:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $token as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_token_t>(),
        );
        $token.type_ = YAML_ANCHOR_TOKEN;
        $token.start_mark = $start_mark;
        $token.end_mark = $end_mark;
        $token.data.anchor.value = $value;
    }};
}

macro_rules! TAG_TOKEN_INIT {
    ($token:expr, $handle:expr, $suffix:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $token as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_token_t>(),
        );
        $token.type_ = YAML_TAG_TOKEN;
        $token.start_mark = $start_mark;
        $token.end_mark = $end_mark;
        $token.data.tag.handle = $handle;
        $token.data.tag.suffix = $suffix;
    }};
}

macro_rules! SCALAR_TOKEN_INIT {
    ($token:expr, $value:expr, $length:expr, $style:expr, $start_mark:expr, $end_mark:expr) => {{
        libc::memset(
            &mut $token as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<yaml_token_t>(),
        );
        $token.type_ = YAML_SCALAR_TOKEN;
        $token.start_mark = $start_mark;
        $token.end_mark = $end_mark;
        $token.data.scalar.value = $value;
        $token.data.scalar.length = $length as size_t;
        $token.data.scalar.style = $style;
    }};
}

const MAX_NUMBER_LENGTH: usize = 9;

#[no_mangle]
pub unsafe extern "C" fn yaml_parser_scan(
    parser: *mut yaml_parser_t,
    token: *mut yaml_token_t,
) -> c_int {
    assert!(!parser.is_null());
    assert!(!token.is_null());

    libc::memset(token as *mut libc::c_void, 0, core::mem::size_of::<yaml_token_t>());

    if (*parser).stream_end_produced != 0 || (*parser).error != YAML_NO_ERROR {
        return 1;
    }

    if (*parser).token_available == 0 {
        if yaml_parser_fetch_more_tokens(parser) == 0 {
            return 0;
        }
    }

    *token = DEQUEUE!(parser, (*parser).tokens);
    (*parser).token_available = 0;
    (*parser).tokens_parsed += 1;

    if (*token).type_ == YAML_STREAM_END_TOKEN {
        (*parser).stream_end_produced = 1;
    }

    1
}

unsafe fn yaml_parser_set_scanner_error(
    parser: *mut yaml_parser_t,
    context: *const c_char,
    context_mark: yaml_mark_t,
    problem: *const c_char,
) -> c_int {
    (*parser).error = YAML_SCANNER_ERROR;
    (*parser).context = context;
    (*parser).context_mark = context_mark;
    (*parser).problem = problem;
    (*parser).problem_mark = (*parser).mark;
    0
}

#[no_mangle]
pub unsafe extern "C" fn yaml_parser_fetch_more_tokens(parser: *mut yaml_parser_t) -> c_int {
    let mut need_more_tokens: c_int;

    loop {
        need_more_tokens = 0;

        if (*parser).tokens.head == (*parser).tokens.tail {
            need_more_tokens = 1;
        } else {
            let mut simple_key: *mut yaml_simple_key_t;

            if yaml_parser_stale_simple_keys(parser) == 0 {
                return 0;
            }

            simple_key = (*parser).simple_keys.start;
            while simple_key != (*parser).simple_keys.top {
                if (*simple_key).possible != 0
                    && (*simple_key).token_number == (*parser).tokens_parsed
                {
                    need_more_tokens = 1;
                    break;
                }
                simple_key = simple_key.add(1);
            }
        }

        if need_more_tokens == 0 {
            break;
        }

        if yaml_parser_fetch_next_token(parser) == 0 {
            return 0;
        }
    }

    (*parser).token_available = 1;
    1
}

unsafe fn yaml_parser_fetch_next_token(parser: *mut yaml_parser_t) -> c_int {
    if CACHE!(parser, 1) == 0 {
        return 0;
    }

    if (*parser).stream_start_produced == 0 {
        return yaml_parser_fetch_stream_start(parser);
    }

    if yaml_parser_scan_to_next_token(parser) == 0 {
        return 0;
    }

    if yaml_parser_stale_simple_keys(parser) == 0 {
        return 0;
    }

    if yaml_parser_unroll_indent(parser, (*parser).mark.column as ptrdiff_t) == 0 {
        return 0;
    }

    if CACHE!(parser, 4) == 0 {
        return 0;
    }

    if IS_Z!((*parser).buffer) {
        return yaml_parser_fetch_stream_end(parser);
    }

    if (*parser).mark.column == 0 && CHECK!((*parser).buffer, b'%') {
        return yaml_parser_fetch_directive(parser);
    }

    if (*parser).mark.column == 0
        && CHECK_AT!((*parser).buffer, b'-', 0)
        && CHECK_AT!((*parser).buffer, b'-', 1)
        && CHECK_AT!((*parser).buffer, b'-', 2)
        && IS_BLANKZ_AT!((*parser).buffer, 3)
    {
        return yaml_parser_fetch_document_indicator(parser, YAML_DOCUMENT_START_TOKEN);
    }

    if (*parser).mark.column == 0
        && CHECK_AT!((*parser).buffer, b'.', 0)
        && CHECK_AT!((*parser).buffer, b'.', 1)
        && CHECK_AT!((*parser).buffer, b'.', 2)
        && IS_BLANKZ_AT!((*parser).buffer, 3)
    {
        return yaml_parser_fetch_document_indicator(parser, YAML_DOCUMENT_END_TOKEN);
    }

    if CHECK!((*parser).buffer, b'[') {
        return yaml_parser_fetch_flow_collection_start(parser, YAML_FLOW_SEQUENCE_START_TOKEN);
    }

    if CHECK!((*parser).buffer, b'{') {
        return yaml_parser_fetch_flow_collection_start(parser, YAML_FLOW_MAPPING_START_TOKEN);
    }

    if CHECK!((*parser).buffer, b']') {
        return yaml_parser_fetch_flow_collection_end(parser, YAML_FLOW_SEQUENCE_END_TOKEN);
    }

    if CHECK!((*parser).buffer, b'}') {
        return yaml_parser_fetch_flow_collection_end(parser, YAML_FLOW_MAPPING_END_TOKEN);
    }

    if CHECK!((*parser).buffer, b',') {
        return yaml_parser_fetch_flow_entry(parser);
    }

    if CHECK!((*parser).buffer, b'-') && IS_BLANKZ_AT!((*parser).buffer, 1) {
        return yaml_parser_fetch_block_entry(parser);
    }

    if CHECK!((*parser).buffer, b'?')
        && ((*parser).flow_level != 0 || IS_BLANKZ_AT!((*parser).buffer, 1))
    {
        return yaml_parser_fetch_key(parser);
    }

    if CHECK!((*parser).buffer, b':')
        && ((*parser).flow_level != 0 || IS_BLANKZ_AT!((*parser).buffer, 1))
    {
        return yaml_parser_fetch_value(parser);
    }

    if CHECK!((*parser).buffer, b'*') {
        return yaml_parser_fetch_anchor(parser, YAML_ALIAS_TOKEN);
    }

    if CHECK!((*parser).buffer, b'&') {
        return yaml_parser_fetch_anchor(parser, YAML_ANCHOR_TOKEN);
    }

    if CHECK!((*parser).buffer, b'!') {
        return yaml_parser_fetch_tag(parser);
    }

    if CHECK!((*parser).buffer, b'|') && (*parser).flow_level == 0 {
        return yaml_parser_fetch_block_scalar(parser, 1);
    }

    if CHECK!((*parser).buffer, b'>') && (*parser).flow_level == 0 {
        return yaml_parser_fetch_block_scalar(parser, 0);
    }

    if CHECK!((*parser).buffer, b'\'') {
        return yaml_parser_fetch_flow_scalar(parser, 1);
    }

    if CHECK!((*parser).buffer, b'"') {
        return yaml_parser_fetch_flow_scalar(parser, 0);
    }

    if !(IS_BLANKZ!((*parser).buffer)
        || CHECK!((*parser).buffer, b'!')
        || CHECK!((*parser).buffer, b'?')
        || CHECK!((*parser).buffer, b':')
        || CHECK!((*parser).buffer, b',')
        || CHECK!((*parser).buffer, b'[')
        || CHECK!((*parser).buffer, b']')
        || CHECK!((*parser).buffer, b'{')
        || CHECK!((*parser).buffer, b'}')
        || CHECK!((*parser).buffer, b'#')
        || CHECK!((*parser).buffer, b'&')
        || CHECK!((*parser).buffer, b'*')
        || CHECK!((*parser).buffer, b'!')
        || CHECK!((*parser).buffer, b'|')
        || CHECK!((*parser).buffer, b'>')
        || CHECK!((*parser).buffer, b'\'')
        || CHECK!((*parser).buffer, b'"')
        || CHECK!((*parser).buffer, b'%')
        || CHECK!((*parser).buffer, b'@')
        || CHECK!((*parser).buffer, b'`'))
        || (CHECK!((*parser).buffer, b'-') && !IS_BLANK_AT!((*parser).buffer, 1))
        || ((*parser).flow_level == 0
            && (CHECK!((*parser).buffer, b'?') || CHECK!((*parser).buffer, b':'))
            && !IS_BLANKZ_AT!((*parser).buffer, 1))
    {
        return yaml_parser_fetch_plain_scalar(parser);
    }

    yaml_parser_set_scanner_error(
        parser,
        b"while scanning for the next token\0".as_ptr() as *const c_char,
        (*parser).mark,
        b"found character that cannot start any token\0".as_ptr() as *const c_char,
    )
}

unsafe fn yaml_parser_stale_simple_keys(parser: *mut yaml_parser_t) -> c_int {
    let mut simple_key: *mut yaml_simple_key_t = (*parser).simple_keys.start;
    while simple_key != (*parser).simple_keys.top {
        if (*simple_key).possible != 0
            && ((*simple_key).mark.line < (*parser).mark.line
                || (*simple_key).mark.index + 1024 < (*parser).mark.index)
        {
            if (*simple_key).required != 0 {
                return yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a simple key\0".as_ptr() as *const c_char,
                    (*simple_key).mark,
                    b"could not find expected ':'\0".as_ptr() as *const c_char,
                );
            }
            (*simple_key).possible = 0;
        }
        simple_key = simple_key.add(1);
    }
    1
}

unsafe fn yaml_parser_save_simple_key(parser: *mut yaml_parser_t) -> c_int {
    let required = if (*parser).flow_level == 0
        && (*parser).indent == (*parser).mark.column as c_int
    {
        1i32
    } else {
        0i32
    };

    if (*parser).simple_key_allowed != 0 {
        let mut simple_key = yaml_simple_key_t {
            possible: 1,
            required,
            token_number: (*parser).tokens_parsed
                + (*parser).tokens.tail.offset_from((*parser).tokens.head) as size_t,
            mark: (*parser).mark,
        };

        if yaml_parser_remove_simple_key(parser) == 0 {
            return 0;
        }

        *((*parser).simple_keys.top.sub(1)) = simple_key;
    }

    1
}

unsafe fn yaml_parser_remove_simple_key(parser: *mut yaml_parser_t) -> c_int {
    let simple_key: *mut yaml_simple_key_t = (*parser).simple_keys.top.sub(1);

    if (*simple_key).possible != 0 {
        if (*simple_key).required != 0 {
            return yaml_parser_set_scanner_error(
                parser,
                b"while scanning a simple key\0".as_ptr() as *const c_char,
                (*simple_key).mark,
                b"could not find expected ':'\0".as_ptr() as *const c_char,
            );
        }
    }

    (*simple_key).possible = 0;
    1
}

unsafe fn yaml_parser_increase_flow_level(parser: *mut yaml_parser_t) -> c_int {
    let empty_simple_key = yaml_simple_key_t {
        possible: 0,
        required: 0,
        token_number: 0,
        mark: yaml_mark_t {
            index: 0,
            line: 0,
            column: 0,
        },
    };

    if PUSH!(parser, (*parser).simple_keys, empty_simple_key) == 0 {
        return 0;
    }

    if (*parser).flow_level == libc::INT_MAX {
        (*parser).error = YAML_MEMORY_ERROR;
        return 0;
    }

    (*parser).flow_level += 1;
    1
}

unsafe fn yaml_parser_decrease_flow_level(parser: *mut yaml_parser_t) -> c_int {
    if (*parser).flow_level != 0 {
        (*parser).flow_level -= 1;
        let _ = POP!(parser, (*parser).simple_keys);
    }
    1
}

unsafe fn yaml_parser_roll_indent(
    parser: *mut yaml_parser_t,
    column: ptrdiff_t,
    number: ptrdiff_t,
    type_: yaml_token_type_t,
    mark: yaml_mark_t,
) -> c_int {
    let mut token: yaml_token_t = core::mem::zeroed();

    if (*parser).flow_level != 0 {
        return 1;
    }

    if ((*parser).indent as ptrdiff_t) < column {
        if PUSH!(parser, (*parser).indents, (*parser).indent) == 0 {
            return 0;
        }

        if column > libc::INT_MAX as ptrdiff_t {
            (*parser).error = YAML_MEMORY_ERROR;
            return 0;
        }

        (*parser).indent = column as c_int;

        TOKEN_INIT!(token, type_, mark, mark);

        if number == -1 {
            if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
                return 0;
            }
        } else {
            if QUEUE_INSERT!(
                parser,
                (*parser).tokens,
                number as size_t - (*parser).tokens_parsed,
                token
            ) == 0
            {
                return 0;
            }
        }
    }

    1
}

unsafe fn yaml_parser_unroll_indent(parser: *mut yaml_parser_t, column: ptrdiff_t) -> c_int {
    let mut token: yaml_token_t = core::mem::zeroed();

    if (*parser).flow_level != 0 {
        return 1;
    }

    while ((*parser).indent as ptrdiff_t) > column {
        TOKEN_INIT!(token, YAML_BLOCK_END_TOKEN, (*parser).mark, (*parser).mark);

        if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
            return 0;
        }

        (*parser).indent = POP!(parser, (*parser).indents);
    }

    1
}

unsafe fn yaml_parser_fetch_stream_start(parser: *mut yaml_parser_t) -> c_int {
    let simple_key = yaml_simple_key_t {
        possible: 0,
        required: 0,
        token_number: 0,
        mark: yaml_mark_t {
            index: 0,
            line: 0,
            column: 0,
        },
    };
    let mut token: yaml_token_t = core::mem::zeroed();

    (*parser).indent = -1;

    if PUSH!(parser, (*parser).simple_keys, simple_key) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 1;
    (*parser).stream_start_produced = 1;

    STREAM_START_TOKEN_INIT!(token, (*parser).encoding, (*parser).mark, (*parser).mark);

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_stream_end(parser: *mut yaml_parser_t) -> c_int {
    let mut token: yaml_token_t = core::mem::zeroed();

    if (*parser).mark.column != 0 {
        (*parser).mark.column = 0;
        (*parser).mark.line += 1;
    }

    if yaml_parser_unroll_indent(parser, -1) == 0 {
        return 0;
    }

    if yaml_parser_remove_simple_key(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 0;

    STREAM_END_TOKEN_INIT!(token, (*parser).mark, (*parser).mark);

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_directive(parser: *mut yaml_parser_t) -> c_int {
    let mut token: yaml_token_t = core::mem::zeroed();

    if yaml_parser_unroll_indent(parser, -1) == 0 {
        return 0;
    }

    if yaml_parser_remove_simple_key(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 0;

    if yaml_parser_scan_directive(parser, &mut token) == 0 {
        return 0;
    }

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        crate::api::yaml_token_delete(&mut token);
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_document_indicator(
    parser: *mut yaml_parser_t,
    type_: yaml_token_type_t,
) -> c_int {
    let start_mark: yaml_mark_t;
    let end_mark: yaml_mark_t;
    let mut token: yaml_token_t = core::mem::zeroed();

    if yaml_parser_unroll_indent(parser, -1) == 0 {
        return 0;
    }

    if yaml_parser_remove_simple_key(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 0;

    start_mark = (*parser).mark;

    SKIP!(parser);
    SKIP!(parser);
    SKIP!(parser);

    end_mark = (*parser).mark;

    TOKEN_INIT!(token, type_, start_mark, end_mark);

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_flow_collection_start(
    parser: *mut yaml_parser_t,
    type_: yaml_token_type_t,
) -> c_int {
    let start_mark: yaml_mark_t;
    let end_mark: yaml_mark_t;
    let mut token: yaml_token_t = core::mem::zeroed();

    if yaml_parser_save_simple_key(parser) == 0 {
        return 0;
    }

    if yaml_parser_increase_flow_level(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 1;

    start_mark = (*parser).mark;
    SKIP!(parser);
    end_mark = (*parser).mark;

    TOKEN_INIT!(token, type_, start_mark, end_mark);

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_flow_collection_end(
    parser: *mut yaml_parser_t,
    type_: yaml_token_type_t,
) -> c_int {
    let start_mark: yaml_mark_t;
    let end_mark: yaml_mark_t;
    let mut token: yaml_token_t = core::mem::zeroed();

    if yaml_parser_remove_simple_key(parser) == 0 {
        return 0;
    }

    if yaml_parser_decrease_flow_level(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 0;

    start_mark = (*parser).mark;
    SKIP!(parser);
    end_mark = (*parser).mark;

    TOKEN_INIT!(token, type_, start_mark, end_mark);

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_flow_entry(parser: *mut yaml_parser_t) -> c_int {
    let start_mark: yaml_mark_t;
    let end_mark: yaml_mark_t;
    let mut token: yaml_token_t = core::mem::zeroed();

    if yaml_parser_remove_simple_key(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 1;

    start_mark = (*parser).mark;
    SKIP!(parser);
    end_mark = (*parser).mark;

    TOKEN_INIT!(token, YAML_FLOW_ENTRY_TOKEN, start_mark, end_mark);

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_block_entry(parser: *mut yaml_parser_t) -> c_int {
    let start_mark: yaml_mark_t;
    let end_mark: yaml_mark_t;
    let mut token: yaml_token_t = core::mem::zeroed();

    if (*parser).flow_level == 0 {
        if (*parser).simple_key_allowed == 0 {
            return yaml_parser_set_scanner_error(
                parser,
                core::ptr::null(),
                (*parser).mark,
                b"block sequence entries are not allowed in this context\0".as_ptr()
                    as *const c_char,
            );
        }

        if yaml_parser_roll_indent(
            parser,
            (*parser).mark.column as ptrdiff_t,
            -1,
            YAML_BLOCK_SEQUENCE_START_TOKEN,
            (*parser).mark,
        ) == 0
        {
            return 0;
        }
    }

    if yaml_parser_remove_simple_key(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 1;

    start_mark = (*parser).mark;
    SKIP!(parser);
    end_mark = (*parser).mark;

    TOKEN_INIT!(token, YAML_BLOCK_ENTRY_TOKEN, start_mark, end_mark);

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_key(parser: *mut yaml_parser_t) -> c_int {
    let start_mark: yaml_mark_t;
    let end_mark: yaml_mark_t;
    let mut token: yaml_token_t = core::mem::zeroed();

    if (*parser).flow_level == 0 {
        if (*parser).simple_key_allowed == 0 {
            return yaml_parser_set_scanner_error(
                parser,
                core::ptr::null(),
                (*parser).mark,
                b"mapping keys are not allowed in this context\0".as_ptr() as *const c_char,
            );
        }

        if yaml_parser_roll_indent(
            parser,
            (*parser).mark.column as ptrdiff_t,
            -1,
            YAML_BLOCK_MAPPING_START_TOKEN,
            (*parser).mark,
        ) == 0
        {
            return 0;
        }
    }

    if yaml_parser_remove_simple_key(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = if (*parser).flow_level == 0 { 1 } else { 0 };

    start_mark = (*parser).mark;
    SKIP!(parser);
    end_mark = (*parser).mark;

    TOKEN_INIT!(token, YAML_KEY_TOKEN, start_mark, end_mark);

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_value(parser: *mut yaml_parser_t) -> c_int {
    let start_mark: yaml_mark_t;
    let end_mark: yaml_mark_t;
    let mut token: yaml_token_t = core::mem::zeroed();
    let simple_key: *mut yaml_simple_key_t = (*parser).simple_keys.top.sub(1);

    if (*simple_key).possible != 0 {
        TOKEN_INIT!(token, YAML_KEY_TOKEN, (*simple_key).mark, (*simple_key).mark);

        if QUEUE_INSERT!(
            parser,
            (*parser).tokens,
            (*simple_key).token_number - (*parser).tokens_parsed,
            token
        ) == 0
        {
            return 0;
        }

        if yaml_parser_roll_indent(
            parser,
            (*simple_key).mark.column as ptrdiff_t,
            (*simple_key).token_number as ptrdiff_t,
            YAML_BLOCK_MAPPING_START_TOKEN,
            (*simple_key).mark,
        ) == 0
        {
            return 0;
        }

        (*simple_key).possible = 0;
        (*parser).simple_key_allowed = 0;
    } else {
        if (*parser).flow_level == 0 {
            if (*parser).simple_key_allowed == 0 {
                return yaml_parser_set_scanner_error(
                    parser,
                    core::ptr::null(),
                    (*parser).mark,
                    b"mapping values are not allowed in this context\0".as_ptr() as *const c_char,
                );
            }

            if yaml_parser_roll_indent(
                parser,
                (*parser).mark.column as ptrdiff_t,
                -1,
                YAML_BLOCK_MAPPING_START_TOKEN,
                (*parser).mark,
            ) == 0
            {
                return 0;
            }
        }

        (*parser).simple_key_allowed = if (*parser).flow_level == 0 { 1 } else { 0 };
    }

    start_mark = (*parser).mark;
    SKIP!(parser);
    end_mark = (*parser).mark;

    TOKEN_INIT!(token, YAML_VALUE_TOKEN, start_mark, end_mark);

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_anchor(
    parser: *mut yaml_parser_t,
    type_: yaml_token_type_t,
) -> c_int {
    let mut token: yaml_token_t = core::mem::zeroed();

    if yaml_parser_save_simple_key(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 0;

    if yaml_parser_scan_anchor(parser, &mut token, type_) == 0 {
        return 0;
    }

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        crate::api::yaml_token_delete(&mut token);
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_tag(parser: *mut yaml_parser_t) -> c_int {
    let mut token: yaml_token_t = core::mem::zeroed();

    if yaml_parser_save_simple_key(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 0;

    if yaml_parser_scan_tag(parser, &mut token) == 0 {
        return 0;
    }

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        crate::api::yaml_token_delete(&mut token);
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_block_scalar(parser: *mut yaml_parser_t, literal: c_int) -> c_int {
    let mut token: yaml_token_t = core::mem::zeroed();

    if yaml_parser_remove_simple_key(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 1;

    if yaml_parser_scan_block_scalar(parser, &mut token, literal) == 0 {
        return 0;
    }

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        crate::api::yaml_token_delete(&mut token);
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_flow_scalar(parser: *mut yaml_parser_t, single: c_int) -> c_int {
    let mut token: yaml_token_t = core::mem::zeroed();

    if yaml_parser_save_simple_key(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 0;

    if yaml_parser_scan_flow_scalar(parser, &mut token, single) == 0 {
        return 0;
    }

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        crate::api::yaml_token_delete(&mut token);
        return 0;
    }

    1
}

unsafe fn yaml_parser_fetch_plain_scalar(parser: *mut yaml_parser_t) -> c_int {
    let mut token: yaml_token_t = core::mem::zeroed();

    if yaml_parser_save_simple_key(parser) == 0 {
        return 0;
    }

    (*parser).simple_key_allowed = 0;

    if yaml_parser_scan_plain_scalar(parser, &mut token) == 0 {
        return 0;
    }

    if ENQUEUE!(parser, (*parser).tokens, token) == 0 {
        crate::api::yaml_token_delete(&mut token);
        return 0;
    }

    1
}

unsafe fn yaml_parser_scan_to_next_token(parser: *mut yaml_parser_t) -> c_int {
    loop {
        if CACHE!(parser, 1) == 0 {
            return 0;
        }

        if (*parser).mark.column == 0 && IS_BOM!((*parser).buffer) {
            SKIP!(parser);
        }

        if CACHE!(parser, 1) == 0 {
            return 0;
        }

        while CHECK!((*parser).buffer, b' ')
            || (((*parser).flow_level != 0 || (*parser).simple_key_allowed == 0)
                && CHECK!((*parser).buffer, b'\t'))
        {
            SKIP!(parser);
            if CACHE!(parser, 1) == 0 {
                return 0;
            }
        }

        if CHECK!((*parser).buffer, b'#') {
            while !IS_BREAKZ!((*parser).buffer) {
                SKIP!(parser);
                if CACHE!(parser, 1) == 0 {
                    return 0;
                }
            }
        }

        if IS_BREAK!((*parser).buffer) {
            if CACHE!(parser, 2) == 0 {
                return 0;
            }
            SKIP_LINE!(parser);

            if (*parser).flow_level == 0 {
                (*parser).simple_key_allowed = 1;
            }
        } else {
            break;
        }
    }

    1
}

unsafe fn yaml_parser_scan_directive(
    parser: *mut yaml_parser_t,
    token: *mut yaml_token_t,
) -> c_int {
    let start_mark: yaml_mark_t;
    let end_mark: yaml_mark_t;
    let mut name: *mut yaml_char_t = core::ptr::null_mut();
    let mut major: c_int = 0;
    let mut minor: c_int = 0;
    let mut handle: *mut yaml_char_t = core::ptr::null_mut();
    let mut prefix: *mut yaml_char_t = core::ptr::null_mut();

    let ok: c_int = 'error: {
        start_mark = (*parser).mark;

        SKIP!(parser);

        if yaml_parser_scan_directive_name(parser, start_mark, &mut name) == 0 {
            break 'error 0;
        }

        if libc::strcmp(name as *const c_char, b"YAML\0".as_ptr() as *const c_char) == 0 {
            if yaml_parser_scan_version_directive_value(parser, start_mark, &mut major, &mut minor)
                == 0
            {
                break 'error 0;
            }

            end_mark = (*parser).mark;

            VERSION_DIRECTIVE_TOKEN_INIT!(*token, major, minor, start_mark, end_mark);
        } else if libc::strcmp(name as *const c_char, b"TAG\0".as_ptr() as *const c_char) == 0 {
            if yaml_parser_scan_tag_directive_value(parser, start_mark, &mut handle, &mut prefix)
                == 0
            {
                break 'error 0;
            }

            end_mark = (*parser).mark;

            TAG_DIRECTIVE_TOKEN_INIT!(*token, handle, prefix, start_mark, end_mark);
        } else {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a directive\0".as_ptr() as *const c_char,
                start_mark,
                b"found unknown directive name\0".as_ptr() as *const c_char,
            );
            break 'error 0;
        }

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        while IS_BLANK!((*parser).buffer) {
            SKIP!(parser);
            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }
        }

        if CHECK!((*parser).buffer, b'#') {
            while !IS_BREAKZ!((*parser).buffer) {
                SKIP!(parser);
                if CACHE!(parser, 1) == 0 {
                    break 'error 0;
                }
            }
        }

        if !IS_BREAKZ!((*parser).buffer) {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a directive\0".as_ptr() as *const c_char,
                start_mark,
                b"did not find expected comment or line break\0".as_ptr() as *const c_char,
            );
            break 'error 0;
        }

        if IS_BREAK!((*parser).buffer) {
            if CACHE!(parser, 2) == 0 {
                break 'error 0;
            }
            SKIP_LINE!(parser);
        }

        crate::api::yaml_free(name as *mut libc::c_void);

        1i32
    };

    if ok == 0 {
        crate::api::yaml_free(prefix as *mut libc::c_void);
        crate::api::yaml_free(handle as *mut libc::c_void);
        crate::api::yaml_free(name as *mut libc::c_void);
        return 0;
    }

    1
}

unsafe fn yaml_parser_scan_directive_name(
    parser: *mut yaml_parser_t,
    start_mark: yaml_mark_t,
    name: *mut *mut yaml_char_t,
) -> c_int {
    let mut string = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };

    let ok: c_int = 'error: {
        if STRING_INIT!(parser, string, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        while IS_ALPHA!((*parser).buffer) {
            if READ!(parser, string) == 0 {
                break 'error 0;
            }
            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }
        }

        if string.start == string.pointer {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a directive\0".as_ptr() as *const c_char,
                start_mark,
                b"could not find expected directive name\0".as_ptr() as *const c_char,
            );
            break 'error 0;
        }

        if !IS_BLANKZ!((*parser).buffer) {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a directive\0".as_ptr() as *const c_char,
                start_mark,
                b"found unexpected non-alphabetical character\0".as_ptr() as *const c_char,
            );
            break 'error 0;
        }

        *name = string.start;
        1i32
    };

    if ok == 0 {
        STRING_DEL!(parser, string);
        return 0;
    }

    1
}

unsafe fn yaml_parser_scan_version_directive_value(
    parser: *mut yaml_parser_t,
    start_mark: yaml_mark_t,
    major: *mut c_int,
    minor: *mut c_int,
) -> c_int {
    if CACHE!(parser, 1) == 0 {
        return 0;
    }

    while IS_BLANK!((*parser).buffer) {
        SKIP!(parser);
        if CACHE!(parser, 1) == 0 {
            return 0;
        }
    }

    if yaml_parser_scan_version_directive_number(parser, start_mark, major) == 0 {
        return 0;
    }

    if !CHECK!((*parser).buffer, b'.') {
        return yaml_parser_set_scanner_error(
            parser,
            b"while scanning a %YAML directive\0".as_ptr() as *const c_char,
            start_mark,
            b"did not find expected digit or '.' character\0".as_ptr() as *const c_char,
        );
    }

    SKIP!(parser);

    if yaml_parser_scan_version_directive_number(parser, start_mark, minor) == 0 {
        return 0;
    }

    1
}

unsafe fn yaml_parser_scan_version_directive_number(
    parser: *mut yaml_parser_t,
    start_mark: yaml_mark_t,
    number: *mut c_int,
) -> c_int {
    let mut value: c_int = 0;
    let mut length: size_t = 0;

    if CACHE!(parser, 1) == 0 {
        return 0;
    }

    while IS_DIGIT!((*parser).buffer) {
        length += 1;
        if length > MAX_NUMBER_LENGTH {
            return yaml_parser_set_scanner_error(
                parser,
                b"while scanning a %YAML directive\0".as_ptr() as *const c_char,
                start_mark,
                b"found extremely long version number\0".as_ptr() as *const c_char,
            );
        }

        value = value * 10 + AS_DIGIT!((*parser).buffer);

        SKIP!(parser);

        if CACHE!(parser, 1) == 0 {
            return 0;
        }
    }

    if length == 0 {
        return yaml_parser_set_scanner_error(
            parser,
            b"while scanning a %YAML directive\0".as_ptr() as *const c_char,
            start_mark,
            b"did not find expected version number\0".as_ptr() as *const c_char,
        );
    }

    *number = value;
    1
}

unsafe fn yaml_parser_scan_tag_directive_value(
    parser: *mut yaml_parser_t,
    start_mark: yaml_mark_t,
    handle: *mut *mut yaml_char_t,
    prefix: *mut *mut yaml_char_t,
) -> c_int {
    let mut handle_value: *mut yaml_char_t = core::ptr::null_mut();
    let mut prefix_value: *mut yaml_char_t = core::ptr::null_mut();

    let ok: c_int = 'error: {
        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        while IS_BLANK!((*parser).buffer) {
            SKIP!(parser);
            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }
        }

        if yaml_parser_scan_tag_handle(parser, 1, start_mark, &mut handle_value) == 0 {
            break 'error 0;
        }

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        if !IS_BLANK!((*parser).buffer) {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a %TAG directive\0".as_ptr() as *const c_char,
                start_mark,
                b"did not find expected whitespace\0".as_ptr() as *const c_char,
            );
            break 'error 0;
        }

        while IS_BLANK!((*parser).buffer) {
            SKIP!(parser);
            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }
        }

        if yaml_parser_scan_tag_uri(
            parser,
            1,
            1,
            core::ptr::null_mut(),
            start_mark,
            &mut prefix_value,
        ) == 0
        {
            break 'error 0;
        }

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        if !IS_BLANKZ!((*parser).buffer) {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a %TAG directive\0".as_ptr() as *const c_char,
                start_mark,
                b"did not find expected whitespace or line break\0".as_ptr() as *const c_char,
            );
            break 'error 0;
        }

        *handle = handle_value;
        *prefix = prefix_value;
        1i32
    };

    if ok == 0 {
        crate::api::yaml_free(handle_value as *mut libc::c_void);
        crate::api::yaml_free(prefix_value as *mut libc::c_void);
        return 0;
    }

    1
}

unsafe fn yaml_parser_scan_anchor(
    parser: *mut yaml_parser_t,
    token: *mut yaml_token_t,
    type_: yaml_token_type_t,
) -> c_int {
    let mut length: c_int = 0;
    let start_mark: yaml_mark_t;
    let end_mark: yaml_mark_t;
    let mut string = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };

    let ok: c_int = 'error: {
        if STRING_INIT!(parser, string, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }

        start_mark = (*parser).mark;

        SKIP!(parser);

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        while IS_ALPHA!((*parser).buffer) {
            if READ!(parser, string) == 0 {
                break 'error 0;
            }
            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }
            length += 1;
        }

        end_mark = (*parser).mark;

        if length == 0
            || !(IS_BLANKZ!((*parser).buffer)
                || CHECK!((*parser).buffer, b'?')
                || CHECK!((*parser).buffer, b':')
                || CHECK!((*parser).buffer, b',')
                || CHECK!((*parser).buffer, b']')
                || CHECK!((*parser).buffer, b'}')
                || CHECK!((*parser).buffer, b'%')
                || CHECK!((*parser).buffer, b'@')
                || CHECK!((*parser).buffer, b'`'))
        {
            yaml_parser_set_scanner_error(
                parser,
                if type_ == YAML_ANCHOR_TOKEN {
                    b"while scanning an anchor\0".as_ptr() as *const c_char
                } else {
                    b"while scanning an alias\0".as_ptr() as *const c_char
                },
                start_mark,
                b"did not find expected alphabetic or numeric character\0".as_ptr()
                    as *const c_char,
            );
            break 'error 0;
        }

        if type_ == YAML_ANCHOR_TOKEN {
            ANCHOR_TOKEN_INIT!(*token, string.start, start_mark, end_mark);
        } else {
            ALIAS_TOKEN_INIT!(*token, string.start, start_mark, end_mark);
        }

        1i32
    };

    if ok == 0 {
        STRING_DEL!(parser, string);
        return 0;
    }

    1
}

unsafe fn yaml_parser_scan_tag(
    parser: *mut yaml_parser_t,
    token: *mut yaml_token_t,
) -> c_int {
    let mut handle: *mut yaml_char_t = core::ptr::null_mut();
    let mut suffix: *mut yaml_char_t = core::ptr::null_mut();
    let start_mark: yaml_mark_t;
    let end_mark: yaml_mark_t;

    let ok: c_int = 'error: {
        start_mark = (*parser).mark;

        if CACHE!(parser, 2) == 0 {
            break 'error 0;
        }

        if CHECK_AT!((*parser).buffer, b'<', 1) {
            handle = YAML_MALLOC(1);
            if handle.is_null() {
                break 'error 0;
            }
            *handle = b'\0';

            SKIP!(parser);
            SKIP!(parser);

            if yaml_parser_scan_tag_uri(
                parser,
                1,
                0,
                core::ptr::null_mut(),
                start_mark,
                &mut suffix,
            ) == 0
            {
                break 'error 0;
            }

            if !CHECK!((*parser).buffer, b'>') {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a tag\0".as_ptr() as *const c_char,
                    start_mark,
                    b"did not find the expected '>'\0".as_ptr() as *const c_char,
                );
                break 'error 0;
            }

            SKIP!(parser);
        } else {
            if yaml_parser_scan_tag_handle(parser, 0, start_mark, &mut handle) == 0 {
                break 'error 0;
            }

            if *handle == b'!'
                && *handle.add(1) != b'\0'
                && *handle.add(libc::strlen(handle as *const c_char) - 1) == b'!'
            {
                if yaml_parser_scan_tag_uri(
                    parser,
                    0,
                    0,
                    core::ptr::null_mut(),
                    start_mark,
                    &mut suffix,
                ) == 0
                {
                    break 'error 0;
                }
            } else {
                if yaml_parser_scan_tag_uri(parser, 0, 0, handle, start_mark, &mut suffix) == 0 {
                    break 'error 0;
                }

                crate::api::yaml_free(handle as *mut libc::c_void);
                handle = YAML_MALLOC(2);
                if handle.is_null() {
                    break 'error 0;
                }
                *handle = b'!';
                *handle.add(1) = b'\0';

                if *suffix == b'\0' {
                    let tmp = handle;
                    handle = suffix;
                    suffix = tmp;
                }
            }
        }

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        if !IS_BLANKZ!((*parser).buffer) {
            if (*parser).flow_level == 0 || !CHECK!((*parser).buffer, b',') {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a tag\0".as_ptr() as *const c_char,
                    start_mark,
                    b"did not find expected whitespace or line break\0".as_ptr() as *const c_char,
                );
                break 'error 0;
            }
        }

        end_mark = (*parser).mark;

        TAG_TOKEN_INIT!(*token, handle, suffix, start_mark, end_mark);
        1i32
    };

    if ok == 0 {
        crate::api::yaml_free(handle as *mut libc::c_void);
        crate::api::yaml_free(suffix as *mut libc::c_void);
        return 0;
    }

    1
}

unsafe fn yaml_parser_scan_tag_handle(
    parser: *mut yaml_parser_t,
    directive: c_int,
    start_mark: yaml_mark_t,
    handle: *mut *mut yaml_char_t,
) -> c_int {
    let mut string = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };

    let ok: c_int = 'error: {
        if STRING_INIT!(parser, string, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        if !CHECK!((*parser).buffer, b'!') {
            yaml_parser_set_scanner_error(
                parser,
                if directive != 0 {
                    b"while scanning a tag directive\0".as_ptr() as *const c_char
                } else {
                    b"while scanning a tag\0".as_ptr() as *const c_char
                },
                start_mark,
                b"did not find expected '!'\0".as_ptr() as *const c_char,
            );
            break 'error 0;
        }

        if READ!(parser, string) == 0 {
            break 'error 0;
        }

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        while IS_ALPHA!((*parser).buffer) {
            if READ!(parser, string) == 0 {
                break 'error 0;
            }
            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }
        }

        if CHECK!((*parser).buffer, b'!') {
            if READ!(parser, string) == 0 {
                break 'error 0;
            }
        } else {
            if directive != 0
                && !(*string.start == b'!' && *string.start.add(1) == b'\0')
            {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while parsing a tag directive\0".as_ptr() as *const c_char,
                    start_mark,
                    b"did not find expected '!'\0".as_ptr() as *const c_char,
                );
                break 'error 0;
            }
        }

        *handle = string.start;
        1i32
    };

    if ok == 0 {
        STRING_DEL!(parser, string);
        return 0;
    }

    1
}

unsafe fn yaml_parser_scan_tag_uri(
    parser: *mut yaml_parser_t,
    uri_char: c_int,
    directive: c_int,
    head: *mut yaml_char_t,
    start_mark: yaml_mark_t,
    uri: *mut *mut yaml_char_t,
) -> c_int {
    let length: size_t = if !head.is_null() {
        libc::strlen(head as *const c_char)
    } else {
        0
    };
    let mut string = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };

    let ok: c_int = 'error: {
        if STRING_INIT!(parser, string, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }

        while (string.end as usize - string.start as usize) <= length {
            if crate::api::yaml_string_extend(
                &mut string.start,
                &mut string.pointer,
                &mut string.end,
            ) == 0
            {
                (*parser).error = YAML_MEMORY_ERROR;
                break 'error 0;
            }
        }

        if length > 1 {
            libc::memcpy(
                string.start as *mut libc::c_void,
                head.add(1) as *const libc::c_void,
                length - 1,
            );
            string.pointer = string.pointer.add(length - 1);
        }

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        let mut cur_length = length;

        loop {
            if !(IS_ALPHA!((*parser).buffer)
                || CHECK!((*parser).buffer, b';')
                || CHECK!((*parser).buffer, b'/')
                || CHECK!((*parser).buffer, b'?')
                || CHECK!((*parser).buffer, b':')
                || CHECK!((*parser).buffer, b'@')
                || CHECK!((*parser).buffer, b'&')
                || CHECK!((*parser).buffer, b'=')
                || CHECK!((*parser).buffer, b'+')
                || CHECK!((*parser).buffer, b'$')
                || CHECK!((*parser).buffer, b'.')
                || CHECK!((*parser).buffer, b'%')
                || CHECK!((*parser).buffer, b'!')
                || CHECK!((*parser).buffer, b'~')
                || CHECK!((*parser).buffer, b'*')
                || CHECK!((*parser).buffer, b'\'')
                || CHECK!((*parser).buffer, b'(')
                || CHECK!((*parser).buffer, b')')
                || (uri_char != 0
                    && (CHECK!((*parser).buffer, b',')
                        || CHECK!((*parser).buffer, b'[')
                        || CHECK!((*parser).buffer, b']'))))
            {
                break;
            }

            if CHECK!((*parser).buffer, b'%') {
                if STRING_EXTEND!(parser, string) == 0 {
                    break 'error 0;
                }

                if yaml_parser_scan_uri_escapes(parser, directive, start_mark, &mut string) == 0 {
                    break 'error 0;
                }
            } else {
                if READ!(parser, string) == 0 {
                    break 'error 0;
                }
            }

            cur_length += 1;
            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }
        }

        if cur_length == 0 {
            if STRING_EXTEND!(parser, string) == 0 {
                break 'error 0;
            }

            yaml_parser_set_scanner_error(
                parser,
                if directive != 0 {
                    b"while parsing a %TAG directive\0".as_ptr() as *const c_char
                } else {
                    b"while parsing a tag\0".as_ptr() as *const c_char
                },
                start_mark,
                b"did not find expected tag URI\0".as_ptr() as *const c_char,
            );
            break 'error 0;
        }

        *uri = string.start;
        1i32
    };

    if ok == 0 {
        STRING_DEL!(parser, string);
        return 0;
    }

    1
}

unsafe fn yaml_parser_scan_uri_escapes(
    parser: *mut yaml_parser_t,
    directive: c_int,
    start_mark: yaml_mark_t,
    string: *mut yaml_string_t,
) -> c_int {
    let mut width: c_int = 0;

    loop {
        let octet: u8;

        if CACHE!(parser, 3) == 0 {
            return 0;
        }

        if !(CHECK!((*parser).buffer, b'%')
            && IS_HEX_AT!((*parser).buffer, 1)
            && IS_HEX_AT!((*parser).buffer, 2))
        {
            return yaml_parser_set_scanner_error(
                parser,
                if directive != 0 {
                    b"while parsing a %TAG directive\0".as_ptr() as *const c_char
                } else {
                    b"while parsing a tag\0".as_ptr() as *const c_char
                },
                start_mark,
                b"did not find URI escaped octet\0".as_ptr() as *const c_char,
            );
        }

        octet = ((AS_HEX_AT!((*parser).buffer, 1) << 4) + AS_HEX_AT!((*parser).buffer, 2)) as u8;

        if width == 0 {
            width = if (octet & 0x80) == 0x00 {
                1
            } else if (octet & 0xE0) == 0xC0 {
                2
            } else if (octet & 0xF0) == 0xE0 {
                3
            } else if (octet & 0xF8) == 0xF0 {
                4
            } else {
                0
            };
            if width == 0 {
                return yaml_parser_set_scanner_error(
                    parser,
                    if directive != 0 {
                        b"while parsing a %TAG directive\0".as_ptr() as *const c_char
                    } else {
                        b"while parsing a tag\0".as_ptr() as *const c_char
                    },
                    start_mark,
                    b"found an incorrect leading UTF-8 octet\0".as_ptr() as *const c_char,
                );
            }
        } else {
            if (octet & 0xC0) != 0x80 {
                return yaml_parser_set_scanner_error(
                    parser,
                    if directive != 0 {
                        b"while parsing a %TAG directive\0".as_ptr() as *const c_char
                    } else {
                        b"while parsing a tag\0".as_ptr() as *const c_char
                    },
                    start_mark,
                    b"found an incorrect trailing UTF-8 octet\0".as_ptr() as *const c_char,
                );
            }
        }

        *(*string).pointer = octet;
        (*string).pointer = (*string).pointer.add(1);
        SKIP!(parser);
        SKIP!(parser);
        SKIP!(parser);

        width -= 1;
        if width == 0 {
            break;
        }
    }

    1
}

unsafe fn yaml_parser_scan_block_scalar(
    parser: *mut yaml_parser_t,
    token: *mut yaml_token_t,
    literal: c_int,
) -> c_int {
    let start_mark: yaml_mark_t;
    let mut end_mark: yaml_mark_t = core::mem::zeroed();
    let mut string = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };
    let mut leading_break = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };
    let mut trailing_breaks = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };
    let mut chomping: c_int = 0;
    let mut increment: c_int = 0;
    let mut indent: c_int = 0;
    let mut leading_blank: c_int = 0;
    let mut trailing_blank: c_int = 0;

    let ok: c_int = 'error: {
        if STRING_INIT!(parser, string, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }
        if STRING_INIT!(parser, leading_break, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }
        if STRING_INIT!(parser, trailing_breaks, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }

        start_mark = (*parser).mark;

        SKIP!(parser);

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        if CHECK!((*parser).buffer, b'+') || CHECK!((*parser).buffer, b'-') {
            chomping = if CHECK!((*parser).buffer, b'+') { 1 } else { -1 };

            SKIP!(parser);

            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }

            if IS_DIGIT!((*parser).buffer) {
                if CHECK!((*parser).buffer, b'0') {
                    yaml_parser_set_scanner_error(
                        parser,
                        b"while scanning a block scalar\0".as_ptr() as *const c_char,
                        start_mark,
                        b"found an indentation indicator equal to 0\0".as_ptr() as *const c_char,
                    );
                    break 'error 0;
                }

                increment = AS_DIGIT!((*parser).buffer);

                SKIP!(parser);
            }
        } else if IS_DIGIT!((*parser).buffer) {
            if CHECK!((*parser).buffer, b'0') {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a block scalar\0".as_ptr() as *const c_char,
                    start_mark,
                    b"found an indentation indicator equal to 0\0".as_ptr() as *const c_char,
                );
                break 'error 0;
            }

            increment = AS_DIGIT!((*parser).buffer);

            SKIP!(parser);

            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }

            if CHECK!((*parser).buffer, b'+') || CHECK!((*parser).buffer, b'-') {
                chomping = if CHECK!((*parser).buffer, b'+') { 1 } else { -1 };

                SKIP!(parser);
            }
        }

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        while IS_BLANK!((*parser).buffer) {
            SKIP!(parser);
            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }
        }

        if CHECK!((*parser).buffer, b'#') {
            while !IS_BREAKZ!((*parser).buffer) {
                SKIP!(parser);
                if CACHE!(parser, 1) == 0 {
                    break 'error 0;
                }
            }
        }

        if !IS_BREAKZ!((*parser).buffer) {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a block scalar\0".as_ptr() as *const c_char,
                start_mark,
                b"did not find expected comment or line break\0".as_ptr() as *const c_char,
            );
            break 'error 0;
        }

        if IS_BREAK!((*parser).buffer) {
            if CACHE!(parser, 2) == 0 {
                break 'error 0;
            }
            SKIP_LINE!(parser);
        }

        end_mark = (*parser).mark;

        if increment != 0 {
            indent = if (*parser).indent >= 0 {
                (*parser).indent as c_int + increment
            } else {
                increment
            };
        }

        if yaml_parser_scan_block_scalar_breaks(
            parser,
            &mut indent,
            &mut trailing_breaks,
            start_mark,
            &mut end_mark,
        ) == 0
        {
            break 'error 0;
        }

        if CACHE!(parser, 1) == 0 {
            break 'error 0;
        }

        while (*parser).mark.column as c_int == indent && !IS_Z!((*parser).buffer) {
            trailing_blank = IS_BLANK!((*parser).buffer) as c_int;

            if literal == 0
                && *leading_break.start == b'\n'
                && leading_blank == 0
                && trailing_blank == 0
            {
                if *trailing_breaks.start == b'\0' {
                    if STRING_EXTEND!(parser, string) == 0 {
                        break 'error 0;
                    }
                    *string.pointer = b' ';
                    string.pointer = string.pointer.add(1);
                }

                CLEAR!(parser, leading_break);
            } else {
                if JOIN!(parser, string, leading_break) == 0 {
                    break 'error 0;
                }
                CLEAR!(parser, leading_break);
            }

            if JOIN!(parser, string, trailing_breaks) == 0 {
                break 'error 0;
            }
            CLEAR!(parser, trailing_breaks);

            leading_blank = IS_BLANK!((*parser).buffer) as c_int;

            while !IS_BREAKZ!((*parser).buffer) {
                if READ!(parser, string) == 0 {
                    break 'error 0;
                }
                if CACHE!(parser, 1) == 0 {
                    break 'error 0;
                }
            }

            if CACHE!(parser, 2) == 0 {
                break 'error 0;
            }

            if READ_LINE!(parser, leading_break) == 0 {
                break 'error 0;
            }

            if yaml_parser_scan_block_scalar_breaks(
                parser,
                &mut indent,
                &mut trailing_breaks,
                start_mark,
                &mut end_mark,
            ) == 0
            {
                break 'error 0;
            }
        }

        if chomping != -1 {
            if JOIN!(parser, string, leading_break) == 0 {
                break 'error 0;
            }
        }
        if chomping == 1 {
            if JOIN!(parser, string, trailing_breaks) == 0 {
                break 'error 0;
            }
        }

        SCALAR_TOKEN_INIT!(
            *token,
            string.start,
            string.pointer.offset_from(string.start) as size_t,
            if literal != 0 {
                YAML_LITERAL_SCALAR_STYLE
            } else {
                YAML_FOLDED_SCALAR_STYLE
            },
            start_mark,
            end_mark
        );

        STRING_DEL!(parser, leading_break);
        STRING_DEL!(parser, trailing_breaks);

        1i32
    };

    if ok == 0 {
        STRING_DEL!(parser, string);
        STRING_DEL!(parser, leading_break);
        STRING_DEL!(parser, trailing_breaks);
        return 0;
    }

    1
}

unsafe fn yaml_parser_scan_block_scalar_breaks(
    parser: *mut yaml_parser_t,
    indent: *mut c_int,
    breaks: *mut yaml_string_t,
    start_mark: yaml_mark_t,
    end_mark: *mut yaml_mark_t,
) -> c_int {
    let mut max_indent: c_int = 0;

    *end_mark = (*parser).mark;

    loop {
        if CACHE!(parser, 1) == 0 {
            return 0;
        }

        while (*indent == 0 || ((*parser).mark.column as c_int) < *indent)
            && IS_SPACE!((*parser).buffer)
        {
            SKIP!(parser);
            if CACHE!(parser, 1) == 0 {
                return 0;
            }
        }

        if (*parser).mark.column as c_int > max_indent {
            max_indent = (*parser).mark.column as c_int;
        }

        if (*indent == 0 || ((*parser).mark.column as c_int) < *indent)
            && IS_TAB!((*parser).buffer)
        {
            return yaml_parser_set_scanner_error(
                parser,
                b"while scanning a block scalar\0".as_ptr() as *const c_char,
                start_mark,
                b"found a tab character where an indentation space is expected\0".as_ptr()
                    as *const c_char,
            );
        }

        if !IS_BREAK!((*parser).buffer) {
            break;
        }

        if CACHE!(parser, 2) == 0 {
            return 0;
        }
        if READ_LINE!(parser, *breaks) == 0 {
            return 0;
        }
        *end_mark = (*parser).mark;
    }

    if *indent == 0 {
        *indent = max_indent;
        if *indent < (*parser).indent as c_int + 1 {
            *indent = (*parser).indent as c_int + 1;
        }
        if *indent < 1 {
            *indent = 1;
        }
    }

    1
}

unsafe fn yaml_parser_scan_flow_scalar(
    parser: *mut yaml_parser_t,
    token: *mut yaml_token_t,
    single: c_int,
) -> c_int {
    let start_mark: yaml_mark_t;
    let end_mark: yaml_mark_t;
    let mut string = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };
    let mut leading_break = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };
    let mut trailing_breaks = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };
    let mut whitespaces = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };
    let mut leading_blanks: c_int;

    let ok: c_int = 'error: {
        if STRING_INIT!(parser, string, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }
        if STRING_INIT!(parser, leading_break, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }
        if STRING_INIT!(parser, trailing_breaks, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }
        if STRING_INIT!(parser, whitespaces, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }

        start_mark = (*parser).mark;

        SKIP!(parser);

        loop {
            if CACHE!(parser, 4) == 0 {
                break 'error 0;
            }

            if (*parser).mark.column == 0
                && ((CHECK_AT!((*parser).buffer, b'-', 0)
                    && CHECK_AT!((*parser).buffer, b'-', 1)
                    && CHECK_AT!((*parser).buffer, b'-', 2))
                    || (CHECK_AT!((*parser).buffer, b'.', 0)
                        && CHECK_AT!((*parser).buffer, b'.', 1)
                        && CHECK_AT!((*parser).buffer, b'.', 2)))
                && IS_BLANKZ_AT!((*parser).buffer, 3)
            {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a quoted scalar\0".as_ptr() as *const c_char,
                    start_mark,
                    b"found unexpected document indicator\0".as_ptr() as *const c_char,
                );
                break 'error 0;
            }

            if IS_Z!((*parser).buffer) {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a quoted scalar\0".as_ptr() as *const c_char,
                    start_mark,
                    b"found unexpected end of stream\0".as_ptr() as *const c_char,
                );
                break 'error 0;
            }

            if CACHE!(parser, 2) == 0 {
                break 'error 0;
            }

            leading_blanks = 0;

            while !IS_BLANKZ!((*parser).buffer) {
                if single != 0
                    && CHECK_AT!((*parser).buffer, b'\'', 0)
                    && CHECK_AT!((*parser).buffer, b'\'', 1)
                {
                    if STRING_EXTEND!(parser, string) == 0 {
                        break 'error 0;
                    }
                    *string.pointer = b'\'';
                    string.pointer = string.pointer.add(1);
                    SKIP!(parser);
                    SKIP!(parser);
                } else if CHECK!((*parser).buffer, if single != 0 { b'\'' } else { b'"' }) {
                    break;
                } else if single == 0
                    && CHECK!((*parser).buffer, b'\\')
                    && IS_BREAK_AT!((*parser).buffer, 1)
                {
                    if CACHE!(parser, 3) == 0 {
                        break 'error 0;
                    }
                    SKIP!(parser);
                    SKIP_LINE!(parser);
                    leading_blanks = 1;
                    break;
                } else if single == 0 && CHECK!((*parser).buffer, b'\\') {
                    let mut code_length: size_t = 0;

                    if STRING_EXTEND!(parser, string) == 0 {
                        break 'error 0;
                    }

                    match *(*parser).buffer.pointer.add(1) {
                        b'0' => {
                            *string.pointer = b'\0';
                            string.pointer = string.pointer.add(1);
                        }
                        b'a' => {
                            *string.pointer = b'\x07';
                            string.pointer = string.pointer.add(1);
                        }
                        b'b' => {
                            *string.pointer = b'\x08';
                            string.pointer = string.pointer.add(1);
                        }
                        b't' | b'\t' => {
                            *string.pointer = b'\x09';
                            string.pointer = string.pointer.add(1);
                        }
                        b'n' => {
                            *string.pointer = b'\x0A';
                            string.pointer = string.pointer.add(1);
                        }
                        b'v' => {
                            *string.pointer = b'\x0B';
                            string.pointer = string.pointer.add(1);
                        }
                        b'f' => {
                            *string.pointer = b'\x0C';
                            string.pointer = string.pointer.add(1);
                        }
                        b'r' => {
                            *string.pointer = b'\x0D';
                            string.pointer = string.pointer.add(1);
                        }
                        b'e' => {
                            *string.pointer = b'\x1B';
                            string.pointer = string.pointer.add(1);
                        }
                        b' ' => {
                            *string.pointer = b'\x20';
                            string.pointer = string.pointer.add(1);
                        }
                        b'"' => {
                            *string.pointer = b'"';
                            string.pointer = string.pointer.add(1);
                        }
                        b'/' => {
                            *string.pointer = b'/';
                            string.pointer = string.pointer.add(1);
                        }
                        b'\\' => {
                            *string.pointer = b'\\';
                            string.pointer = string.pointer.add(1);
                        }
                        b'N' => {
                            *string.pointer = 0xC2u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = 0x85u8;
                            string.pointer = string.pointer.add(1);
                        }
                        b'_' => {
                            *string.pointer = 0xC2u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = 0xA0u8;
                            string.pointer = string.pointer.add(1);
                        }
                        b'L' => {
                            *string.pointer = 0xE2u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = 0x80u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = 0xA8u8;
                            string.pointer = string.pointer.add(1);
                        }
                        b'P' => {
                            *string.pointer = 0xE2u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = 0x80u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = 0xA9u8;
                            string.pointer = string.pointer.add(1);
                        }
                        b'x' => {
                            code_length = 2;
                        }
                        b'u' => {
                            code_length = 4;
                        }
                        b'U' => {
                            code_length = 8;
                        }
                        _ => {
                            yaml_parser_set_scanner_error(
                                parser,
                                b"while parsing a quoted scalar\0".as_ptr() as *const c_char,
                                start_mark,
                                b"found unknown escape character\0".as_ptr() as *const c_char,
                            );
                            break 'error 0;
                        }
                    }

                    SKIP!(parser);
                    SKIP!(parser);

                    if code_length != 0 {
                        let mut value: u32 = 0;

                        if CACHE!(parser, code_length) == 0 {
                            break 'error 0;
                        }

                        for k in 0..code_length {
                            if !IS_HEX_AT!((*parser).buffer, k) {
                                yaml_parser_set_scanner_error(
                                    parser,
                                    b"while parsing a quoted scalar\0".as_ptr() as *const c_char,
                                    start_mark,
                                    b"did not find expected hexdecimal number\0".as_ptr()
                                        as *const c_char,
                                );
                                break 'error 0;
                            }
                            value = (value << 4) + AS_HEX_AT!((*parser).buffer, k) as u32;
                        }

                        if (value >= 0xD800 && value <= 0xDFFF) || value > 0x10FFFF {
                            yaml_parser_set_scanner_error(
                                parser,
                                b"while parsing a quoted scalar\0".as_ptr() as *const c_char,
                                start_mark,
                                b"found invalid Unicode character escape code\0".as_ptr()
                                    as *const c_char,
                            );
                            break 'error 0;
                        }

                        if value <= 0x7F {
                            *string.pointer = value as u8;
                            string.pointer = string.pointer.add(1);
                        } else if value <= 0x7FF {
                            *string.pointer = (0xC0 + (value >> 6)) as u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = (0x80 + (value & 0x3F)) as u8;
                            string.pointer = string.pointer.add(1);
                        } else if value <= 0xFFFF {
                            *string.pointer = (0xE0 + (value >> 12)) as u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = (0x80 + ((value >> 6) & 0x3F)) as u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = (0x80 + (value & 0x3F)) as u8;
                            string.pointer = string.pointer.add(1);
                        } else {
                            *string.pointer = (0xF0 + (value >> 18)) as u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = (0x80 + ((value >> 12) & 0x3F)) as u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = (0x80 + ((value >> 6) & 0x3F)) as u8;
                            string.pointer = string.pointer.add(1);
                            *string.pointer = (0x80 + (value & 0x3F)) as u8;
                            string.pointer = string.pointer.add(1);
                        }

                        for _ in 0..code_length {
                            SKIP!(parser);
                        }
                    }
                } else {
                    if READ!(parser, string) == 0 {
                        break 'error 0;
                    }
                }

                if CACHE!(parser, 2) == 0 {
                    break 'error 0;
                }
            }

            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }
            if CHECK!((*parser).buffer, if single != 0 { b'\'' } else { b'"' }) {
                break;
            }

            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }

            while IS_BLANK!((*parser).buffer) || IS_BREAK!((*parser).buffer) {
                if IS_BLANK!((*parser).buffer) {
                    if leading_blanks == 0 {
                        if READ!(parser, whitespaces) == 0 {
                            break 'error 0;
                        }
                    } else {
                        SKIP!(parser);
                    }
                } else {
                    if CACHE!(parser, 2) == 0 {
                        break 'error 0;
                    }

                    if leading_blanks == 0 {
                        CLEAR!(parser, whitespaces);
                        if READ_LINE!(parser, leading_break) == 0 {
                            break 'error 0;
                        }
                        leading_blanks = 1;
                    } else {
                        if READ_LINE!(parser, trailing_breaks) == 0 {
                            break 'error 0;
                        }
                    }
                }
                if CACHE!(parser, 1) == 0 {
                    break 'error 0;
                }
            }

            if leading_blanks != 0 {
                if *leading_break.start == b'\n' {
                    if *trailing_breaks.start == b'\0' {
                        if STRING_EXTEND!(parser, string) == 0 {
                            break 'error 0;
                        }
                        *string.pointer = b' ';
                        string.pointer = string.pointer.add(1);
                    } else {
                        if JOIN!(parser, string, trailing_breaks) == 0 {
                            break 'error 0;
                        }
                        CLEAR!(parser, trailing_breaks);
                    }
                    CLEAR!(parser, leading_break);
                } else {
                    if JOIN!(parser, string, leading_break) == 0 {
                        break 'error 0;
                    }
                    if JOIN!(parser, string, trailing_breaks) == 0 {
                        break 'error 0;
                    }
                    CLEAR!(parser, leading_break);
                    CLEAR!(parser, trailing_breaks);
                }
            } else {
                if JOIN!(parser, string, whitespaces) == 0 {
                    break 'error 0;
                }
                CLEAR!(parser, whitespaces);
            }
        }

        SKIP!(parser);

        end_mark = (*parser).mark;

        SCALAR_TOKEN_INIT!(
            *token,
            string.start,
            string.pointer.offset_from(string.start) as size_t,
            if single != 0 {
                YAML_SINGLE_QUOTED_SCALAR_STYLE
            } else {
                YAML_DOUBLE_QUOTED_SCALAR_STYLE
            },
            start_mark,
            end_mark
        );

        STRING_DEL!(parser, leading_break);
        STRING_DEL!(parser, trailing_breaks);
        STRING_DEL!(parser, whitespaces);

        1i32
    };

    if ok == 0 {
        STRING_DEL!(parser, string);
        STRING_DEL!(parser, leading_break);
        STRING_DEL!(parser, trailing_breaks);
        STRING_DEL!(parser, whitespaces);
        return 0;
    }

    1
}

unsafe fn yaml_parser_scan_plain_scalar(
    parser: *mut yaml_parser_t,
    token: *mut yaml_token_t,
) -> c_int {
    let start_mark: yaml_mark_t;
    let mut end_mark: yaml_mark_t;
    let mut string = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };
    let mut leading_break = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };
    let mut trailing_breaks = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };
    let mut whitespaces = yaml_string_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        pointer: core::ptr::null_mut(),
    };
    let mut leading_blanks: c_int = 0;
    let indent: c_int = (*parser).indent as c_int + 1;

    let ok: c_int = 'error: {
        if STRING_INIT!(parser, string, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }
        if STRING_INIT!(parser, leading_break, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }
        if STRING_INIT!(parser, trailing_breaks, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }
        if STRING_INIT!(parser, whitespaces, INITIAL_STRING_SIZE) == 0 {
            break 'error 0;
        }

        start_mark = (*parser).mark;
        end_mark = (*parser).mark;

        loop {
            if CACHE!(parser, 4) == 0 {
                break 'error 0;
            }

            if (*parser).mark.column == 0
                && ((CHECK_AT!((*parser).buffer, b'-', 0)
                    && CHECK_AT!((*parser).buffer, b'-', 1)
                    && CHECK_AT!((*parser).buffer, b'-', 2))
                    || (CHECK_AT!((*parser).buffer, b'.', 0)
                        && CHECK_AT!((*parser).buffer, b'.', 1)
                        && CHECK_AT!((*parser).buffer, b'.', 2)))
                && IS_BLANKZ_AT!((*parser).buffer, 3)
            {
                break;
            }

            if CHECK!((*parser).buffer, b'#') {
                break;
            }

            while !IS_BLANKZ!((*parser).buffer) {
                if (*parser).flow_level != 0
                    && CHECK!((*parser).buffer, b':')
                    && (CHECK_AT!((*parser).buffer, b',', 1)
                        || CHECK_AT!((*parser).buffer, b'?', 1)
                        || CHECK_AT!((*parser).buffer, b'[', 1)
                        || CHECK_AT!((*parser).buffer, b']', 1)
                        || CHECK_AT!((*parser).buffer, b'{', 1)
                        || CHECK_AT!((*parser).buffer, b'}', 1))
                {
                    yaml_parser_set_scanner_error(
                        parser,
                        b"while scanning a plain scalar\0".as_ptr() as *const c_char,
                        start_mark,
                        b"found unexpected ':'\0".as_ptr() as *const c_char,
                    );
                    break 'error 0;
                }

                if (CHECK!((*parser).buffer, b':') && IS_BLANKZ_AT!((*parser).buffer, 1))
                    || ((*parser).flow_level != 0
                        && (CHECK!((*parser).buffer, b',')
                            || CHECK!((*parser).buffer, b'[')
                            || CHECK!((*parser).buffer, b']')
                            || CHECK!((*parser).buffer, b'{')
                            || CHECK!((*parser).buffer, b'}')))
                {
                    break;
                }

                if leading_blanks != 0 || whitespaces.start != whitespaces.pointer {
                    if leading_blanks != 0 {
                        if *leading_break.start == b'\n' {
                            if *trailing_breaks.start == b'\0' {
                                if STRING_EXTEND!(parser, string) == 0 {
                                    break 'error 0;
                                }
                                *string.pointer = b' ';
                                string.pointer = string.pointer.add(1);
                            } else {
                                if JOIN!(parser, string, trailing_breaks) == 0 {
                                    break 'error 0;
                                }
                                CLEAR!(parser, trailing_breaks);
                            }
                            CLEAR!(parser, leading_break);
                        } else {
                            if JOIN!(parser, string, leading_break) == 0 {
                                break 'error 0;
                            }
                            if JOIN!(parser, string, trailing_breaks) == 0 {
                                break 'error 0;
                            }
                            CLEAR!(parser, leading_break);
                            CLEAR!(parser, trailing_breaks);
                        }

                        leading_blanks = 0;
                    } else {
                        if JOIN!(parser, string, whitespaces) == 0 {
                            break 'error 0;
                        }
                        CLEAR!(parser, whitespaces);
                    }
                }

                if READ!(parser, string) == 0 {
                    break 'error 0;
                }

                end_mark = (*parser).mark;

                if CACHE!(parser, 2) == 0 {
                    break 'error 0;
                }
            }

            if !(IS_BLANK!((*parser).buffer) || IS_BREAK!((*parser).buffer)) {
                break;
            }

            if CACHE!(parser, 1) == 0 {
                break 'error 0;
            }

            while IS_BLANK!((*parser).buffer) || IS_BREAK!((*parser).buffer) {
                if IS_BLANK!((*parser).buffer) {
                    if leading_blanks != 0
                        && ((*parser).mark.column as c_int) < indent
                        && IS_TAB!((*parser).buffer)
                    {
                        yaml_parser_set_scanner_error(
                            parser,
                            b"while scanning a plain scalar\0".as_ptr() as *const c_char,
                            start_mark,
                            b"found a tab character that violates indentation\0".as_ptr()
                                as *const c_char,
                        );
                        break 'error 0;
                    }

                    if leading_blanks == 0 {
                        if READ!(parser, whitespaces) == 0 {
                            break 'error 0;
                        }
                    } else {
                        SKIP!(parser);
                    }
                } else {
                    if CACHE!(parser, 2) == 0 {
                        break 'error 0;
                    }

                    if leading_blanks == 0 {
                        CLEAR!(parser, whitespaces);
                        if READ_LINE!(parser, leading_break) == 0 {
                            break 'error 0;
                        }
                        leading_blanks = 1;
                    } else {
                        if READ_LINE!(parser, trailing_breaks) == 0 {
                            break 'error 0;
                        }
                    }
                }
                if CACHE!(parser, 1) == 0 {
                    break 'error 0;
                }
            }

            if (*parser).flow_level == 0 && ((*parser).mark.column as c_int) < indent {
                break;
            }
        }

        SCALAR_TOKEN_INIT!(
            *token,
            string.start,
            string.pointer.offset_from(string.start) as size_t,
            YAML_PLAIN_SCALAR_STYLE,
            start_mark,
            end_mark
        );

        if leading_blanks != 0 {
            (*parser).simple_key_allowed = 1;
        }

        STRING_DEL!(parser, leading_break);
        STRING_DEL!(parser, trailing_breaks);
        STRING_DEL!(parser, whitespaces);

        1i32
    };

    if ok == 0 {
        STRING_DEL!(parser, string);
        STRING_DEL!(parser, leading_break);
        STRING_DEL!(parser, trailing_breaks);
        STRING_DEL!(parser, whitespaces);
        return 0;
    }

    1
}
