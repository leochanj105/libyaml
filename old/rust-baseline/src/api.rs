use crate::*;
use libc::{c_char, c_int, c_void, size_t, FILE};

// Tag string constants
pub const YAML_NULL_TAG: &[u8] = b"tag:yaml.org,2002:null\0";
pub const YAML_BOOL_TAG: &[u8] = b"tag:yaml.org,2002:bool\0";
pub const YAML_STR_TAG: &[u8] = b"tag:yaml.org,2002:str\0";
pub const YAML_INT_TAG: &[u8] = b"tag:yaml.org,2002:int\0";
pub const YAML_FLOAT_TAG: &[u8] = b"tag:yaml.org,2002:float\0";
pub const YAML_TIMESTAMP_TAG: &[u8] = b"tag:yaml.org,2002:timestamp\0";
pub const YAML_SEQ_TAG: &[u8] = b"tag:yaml.org,2002:seq\0";
pub const YAML_MAP_TAG: &[u8] = b"tag:yaml.org,2002:map\0";
pub const YAML_DEFAULT_SCALAR_TAG: &[u8] = b"tag:yaml.org,2002:str\0";
pub const YAML_DEFAULT_SEQUENCE_TAG: &[u8] = b"tag:yaml.org,2002:seq\0";
pub const YAML_DEFAULT_MAPPING_TAG: &[u8] = b"tag:yaml.org,2002:map\0";

// Minimal local error context for API functions that need one
#[repr(C)]
pub struct ErrContext {
    pub error: yaml_error_type_t,
}

// Inline stack types used locally in API functions
pub struct LocalTagDirStack {
    pub start: *mut yaml_tag_directive_t,
    pub end: *mut yaml_tag_directive_t,
    pub top: *mut yaml_tag_directive_t,
}

pub struct LocalNodeStack {
    pub start: *mut yaml_node_t,
    pub end: *mut yaml_node_t,
    pub top: *mut yaml_node_t,
}

pub struct LocalNodeItemStack {
    pub start: *mut yaml_node_item_t,
    pub end: *mut yaml_node_item_t,
    pub top: *mut yaml_node_item_t,
}

pub struct LocalNodePairStack {
    pub start: *mut yaml_node_pair_t,
    pub end: *mut yaml_node_pair_t,
    pub top: *mut yaml_node_pair_t,
}

// Macros for event/token/document/node init (from yaml_private.h)
macro_rules! STREAM_START_EVENT_INIT {
    ($event:expr, $encoding:expr, $start_mark:expr, $end_mark:expr) => {
        EVENT_INIT!($event, YAML_STREAM_START_EVENT, $start_mark, $end_mark);
        $event.data.stream_start.encoding = $encoding;
    };
}

macro_rules! STREAM_END_EVENT_INIT {
    ($event:expr, $start_mark:expr, $end_mark:expr) => {
        EVENT_INIT!($event, YAML_STREAM_END_EVENT, $start_mark, $end_mark);
    };
}

macro_rules! DOCUMENT_START_EVENT_INIT {
    ($event:expr, $version_directive:expr, $tag_dir_start:expr, $tag_dir_end:expr, $implicit:expr, $start_mark:expr, $end_mark:expr) => {
        EVENT_INIT!($event, YAML_DOCUMENT_START_EVENT, $start_mark, $end_mark);
        $event.data.document_start.version_directive = $version_directive;
        $event.data.document_start.tag_directives.start = $tag_dir_start;
        $event.data.document_start.tag_directives.end = $tag_dir_end;
        $event.data.document_start.implicit = $implicit;
    };
}

macro_rules! DOCUMENT_END_EVENT_INIT {
    ($event:expr, $implicit:expr, $start_mark:expr, $end_mark:expr) => {
        EVENT_INIT!($event, YAML_DOCUMENT_END_EVENT, $start_mark, $end_mark);
        $event.data.document_end.implicit = $implicit;
    };
}

macro_rules! ALIAS_EVENT_INIT {
    ($event:expr, $anchor:expr, $start_mark:expr, $end_mark:expr) => {
        EVENT_INIT!($event, YAML_ALIAS_EVENT, $start_mark, $end_mark);
        $event.data.alias.anchor = $anchor;
    };
}

macro_rules! SCALAR_EVENT_INIT {
    ($event:expr, $anchor:expr, $tag:expr, $value:expr, $length:expr,
     $plain_implicit:expr, $quoted_implicit:expr, $style:expr, $start_mark:expr, $end_mark:expr) => {
        EVENT_INIT!($event, YAML_SCALAR_EVENT, $start_mark, $end_mark);
        $event.data.scalar.anchor = $anchor;
        $event.data.scalar.tag = $tag;
        $event.data.scalar.value = $value;
        $event.data.scalar.length = $length as size_t;
        $event.data.scalar.plain_implicit = $plain_implicit;
        $event.data.scalar.quoted_implicit = $quoted_implicit;
        $event.data.scalar.style = $style;
    };
}

macro_rules! SEQUENCE_START_EVENT_INIT {
    ($event:expr, $anchor:expr, $tag:expr, $implicit:expr, $style:expr, $start_mark:expr, $end_mark:expr) => {
        EVENT_INIT!($event, YAML_SEQUENCE_START_EVENT, $start_mark, $end_mark);
        $event.data.sequence_start.anchor = $anchor;
        $event.data.sequence_start.tag = $tag;
        $event.data.sequence_start.implicit = $implicit;
        $event.data.sequence_start.style = $style;
    };
}

macro_rules! SEQUENCE_END_EVENT_INIT {
    ($event:expr, $start_mark:expr, $end_mark:expr) => {
        EVENT_INIT!($event, YAML_SEQUENCE_END_EVENT, $start_mark, $end_mark);
    };
}

macro_rules! MAPPING_START_EVENT_INIT {
    ($event:expr, $anchor:expr, $tag:expr, $implicit:expr, $style:expr, $start_mark:expr, $end_mark:expr) => {
        EVENT_INIT!($event, YAML_MAPPING_START_EVENT, $start_mark, $end_mark);
        $event.data.mapping_start.anchor = $anchor;
        $event.data.mapping_start.tag = $tag;
        $event.data.mapping_start.implicit = $implicit;
        $event.data.mapping_start.style = $style;
    };
}

macro_rules! MAPPING_END_EVENT_INIT {
    ($event:expr, $start_mark:expr, $end_mark:expr) => {
        EVENT_INIT!($event, YAML_MAPPING_END_EVENT, $start_mark, $end_mark);
    };
}

macro_rules! NODE_INIT {
    ($node:expr, $node_type:expr, $node_tag:expr, $start_mark:expr, $end_mark:expr) => {
        libc::memset(
            &mut $node as *mut _ as *mut c_void,
            0,
            core::mem::size_of::<yaml_node_t>(),
        );
        $node.type_ = $node_type;
        $node.tag = $node_tag;
        $node.start_mark = $start_mark;
        $node.end_mark = $end_mark;
    };
}

macro_rules! SCALAR_NODE_INIT {
    ($node:expr, $tag:expr, $value:expr, $length:expr, $style:expr, $start_mark:expr, $end_mark:expr) => {
        NODE_INIT!($node, YAML_SCALAR_NODE, $tag, $start_mark, $end_mark);
        $node.data.scalar.value = $value;
        $node.data.scalar.length = $length as size_t;
        $node.data.scalar.style = $style;
    };
}

macro_rules! SEQUENCE_NODE_INIT {
    ($node:expr, $tag:expr, $items_start:expr, $items_end:expr, $style:expr, $start_mark:expr, $end_mark:expr) => {
        NODE_INIT!($node, YAML_SEQUENCE_NODE, $tag, $start_mark, $end_mark);
        $node.data.sequence.items.start = $items_start;
        $node.data.sequence.items.end = $items_end;
        $node.data.sequence.items.top = $items_start;
        $node.data.sequence.style = $style;
    };
}

macro_rules! MAPPING_NODE_INIT {
    ($node:expr, $tag:expr, $pairs_start:expr, $pairs_end:expr, $style:expr, $start_mark:expr, $end_mark:expr) => {
        NODE_INIT!($node, YAML_MAPPING_NODE, $tag, $start_mark, $end_mark);
        $node.data.mapping.pairs.start = $pairs_start;
        $node.data.mapping.pairs.end = $pairs_end;
        $node.data.mapping.pairs.top = $pairs_start;
        $node.data.mapping.style = $style;
    };
}

macro_rules! DOCUMENT_INIT {
    ($document:expr, $nodes_start:expr, $nodes_end:expr, $version_directive:expr,
     $tag_dir_start:expr, $tag_dir_end:expr,
     $start_implicit:expr, $end_implicit:expr, $start_mark:expr, $end_mark:expr) => {
        libc::memset(
            &mut $document as *mut _ as *mut c_void,
            0,
            core::mem::size_of::<yaml_document_t>(),
        );
        $document.nodes.start = $nodes_start;
        $document.nodes.end = $nodes_end;
        $document.nodes.top = $nodes_start;
        $document.version_directive = $version_directive;
        $document.tag_directives.start = $tag_dir_start;
        $document.tag_directives.end = $tag_dir_end;
        $document.start_implicit = $start_implicit;
        $document.end_implicit = $end_implicit;
        $document.start_mark = $start_mark;
        $document.end_mark = $end_mark;
    };
}

// ---- Public API functions ----

#[no_mangle]
pub extern "C" fn yaml_get_version_string() -> *const c_char {
    YAML_VERSION_STRING.as_ptr() as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn yaml_get_version(major: *mut c_int, minor: *mut c_int, patch: *mut c_int) {
    *major = YAML_VERSION_MAJOR;
    *minor = YAML_VERSION_MINOR;
    *patch = YAML_VERSION_PATCH;
}

#[no_mangle]
pub unsafe extern "C" fn yaml_malloc(size: size_t) -> *mut c_void {
    libc::malloc(if size != 0 { size } else { 1 })
}

#[no_mangle]
pub unsafe extern "C" fn yaml_realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    if !ptr.is_null() {
        libc::realloc(ptr, if size != 0 { size } else { 1 })
    } else {
        libc::malloc(if size != 0 { size } else { 1 })
    }
}

#[no_mangle]
pub unsafe extern "C" fn yaml_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        libc::free(ptr)
    }
}

#[no_mangle]
pub unsafe extern "C" fn yaml_strdup(str_: *const yaml_char_t) -> *mut yaml_char_t {
    if str_.is_null() {
        return core::ptr::null_mut();
    }
    libc::strdup(str_ as *const c_char) as *mut yaml_char_t
}

#[no_mangle]
pub unsafe extern "C" fn yaml_string_extend(
    start: *mut *mut yaml_char_t,
    pointer: *mut *mut yaml_char_t,
    end: *mut *mut yaml_char_t,
) -> c_int {
    let old_size = (*end).offset_from(*start) as usize;
    let new_start =
        yaml_realloc(*start as *mut c_void, old_size * 2) as *mut yaml_char_t;

    if new_start.is_null() {
        return 0;
    }

    libc::memset(new_start.add(old_size) as *mut c_void, 0, old_size);

    *pointer = new_start.add((*pointer).offset_from(*start) as usize);
    *end = new_start.add(old_size * 2);
    *start = new_start;

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_string_join(
    a_start: *mut *mut yaml_char_t,
    a_pointer: *mut *mut yaml_char_t,
    a_end: *mut *mut yaml_char_t,
    b_start: *mut *mut yaml_char_t,
    b_pointer: *mut *mut yaml_char_t,
    b_end: *mut *mut yaml_char_t,
) -> c_int {
    let _ = b_end; // UNUSED_PARAM
    if *b_start == *b_pointer {
        return 1;
    }

    while (*a_end).offset_from(*a_pointer) <= (*b_pointer).offset_from(*b_start) {
        if yaml_string_extend(a_start, a_pointer, a_end) == 0 {
            return 0;
        }
    }

    let b_len = (*b_pointer).offset_from(*b_start) as usize;
    libc::memcpy(*a_pointer as *mut c_void, *b_start as *const c_void, b_len);
    *a_pointer = (*a_pointer).add(b_len);

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_stack_extend(
    start: *mut *mut c_void,
    top: *mut *mut c_void,
    end: *mut *mut c_void,
) -> c_int {
    let old_size = (*end as *mut u8).offset_from(*start as *mut u8) as usize;
    if old_size >= (libc::INT_MAX as usize) / 2 {
        return 0;
    }

    let new_start = yaml_realloc(*start, old_size * 2);
    if new_start.is_null() {
        return 0;
    }

    *top = (new_start as *mut u8)
        .add((*top as *mut u8).offset_from(*start as *mut u8) as usize)
        as *mut c_void;
    *end = (new_start as *mut u8).add(old_size * 2) as *mut c_void;
    *start = new_start;

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_queue_extend(
    start: *mut *mut c_void,
    head: *mut *mut c_void,
    tail: *mut *mut c_void,
    end: *mut *mut c_void,
) -> c_int {
    // Check if we need to resize
    if *start == *head && *tail == *end {
        let old_size = (*end as *mut u8).offset_from(*start as *mut u8) as usize;
        let new_start = yaml_realloc(*start, old_size * 2);
        if new_start.is_null() {
            return 0;
        }
        *head = (new_start as *mut u8)
            .add((*head as *mut u8).offset_from(*start as *mut u8) as usize)
            as *mut c_void;
        *tail = (new_start as *mut u8)
            .add((*tail as *mut u8).offset_from(*start as *mut u8) as usize)
            as *mut c_void;
        *end = (new_start as *mut u8).add(old_size * 2) as *mut c_void;
        *start = new_start;
    }

    // Check if we need to move queue to beginning
    if *tail == *end {
        if *head != *tail {
            let len = (*tail as *mut u8).offset_from(*head as *mut u8) as usize;
            libc::memmove(*start, *head as *const c_void, len);
        }
        let tail_offset = (*tail as *mut u8).offset_from(*head as *mut u8) as usize;
        *tail = (*start as *mut u8).add(tail_offset) as *mut c_void;
        *head = *start;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_parser_initialize(parser: *mut yaml_parser_t) -> c_int {
    debug_assert!(!parser.is_null());

    libc::memset(parser as *mut c_void, 0, core::mem::size_of::<yaml_parser_t>());

    let ok: c_int = 'error: {
        if BUFFER_INIT!(parser, (*parser).raw_buffer, INPUT_RAW_BUFFER_SIZE) == 0 {
            break 'error 0;
        }
        if BUFFER_INIT!(parser, (*parser).buffer, INPUT_BUFFER_SIZE) == 0 {
            break 'error 0;
        }
        if QUEUE_INIT!(parser, (*parser).tokens, INITIAL_QUEUE_SIZE, yaml_token_t) == 0 {
            break 'error 0;
        }
        if STACK_INIT!(parser, (*parser).indents, c_int) == 0 {
            break 'error 0;
        }
        if STACK_INIT!(parser, (*parser).simple_keys, yaml_simple_key_t) == 0 {
            break 'error 0;
        }
        if STACK_INIT!(parser, (*parser).states, yaml_parser_state_t) == 0 {
            break 'error 0;
        }
        if STACK_INIT!(parser, (*parser).marks, yaml_mark_t) == 0 {
            break 'error 0;
        }
        if STACK_INIT!(parser, (*parser).tag_directives, yaml_tag_directive_t) == 0 {
            break 'error 0;
        }
        1
    };

    if ok == 0 {
        BUFFER_DEL!(parser, (*parser).raw_buffer);
        BUFFER_DEL!(parser, (*parser).buffer);
        QUEUE_DEL!(parser, (*parser).tokens);
        STACK_DEL!(parser, (*parser).indents);
        STACK_DEL!(parser, (*parser).simple_keys);
        STACK_DEL!(parser, (*parser).states);
        STACK_DEL!(parser, (*parser).marks);
        STACK_DEL!(parser, (*parser).tag_directives);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_parser_delete(parser: *mut yaml_parser_t) {
    debug_assert!(!parser.is_null());

    BUFFER_DEL!(parser, (*parser).raw_buffer);
    BUFFER_DEL!(parser, (*parser).buffer);
    while !QUEUE_EMPTY!(parser, (*parser).tokens) {
        let mut tok = DEQUEUE!(parser, (*parser).tokens);
        yaml_token_delete(&mut tok);
    }
    QUEUE_DEL!(parser, (*parser).tokens);
    STACK_DEL!(parser, (*parser).indents);
    STACK_DEL!(parser, (*parser).simple_keys);
    STACK_DEL!(parser, (*parser).states);
    STACK_DEL!(parser, (*parser).marks);
    while !STACK_EMPTY!(parser, (*parser).tag_directives) {
        let tag_directive = POP!(parser, (*parser).tag_directives);
        yaml_free(tag_directive.handle as *mut c_void);
        yaml_free(tag_directive.prefix as *mut c_void);
    }
    STACK_DEL!(parser, (*parser).tag_directives);

    libc::memset(parser as *mut c_void, 0, core::mem::size_of::<yaml_parser_t>());
}

unsafe extern "C" fn yaml_string_read_handler(
    data: *mut c_void,
    buffer: *mut u8,
    size: size_t,
    size_read: *mut size_t,
) -> c_int {
    let parser = data as *mut yaml_parser_t;

    if (*parser).input.string.current == (*parser).input.string.end {
        *size_read = 0;
        return 1;
    }

    let mut sz = size;
    let avail = (*parser)
        .input
        .string
        .end
        .offset_from((*parser).input.string.current) as usize;
    if sz > avail {
        sz = avail;
    }

    libc::memcpy(buffer as *mut c_void, (*parser).input.string.current as *const c_void, sz);
    (*parser).input.string.current = (*parser).input.string.current.add(sz);
    *size_read = sz;
    1
}

unsafe extern "C" fn yaml_file_read_handler(
    data: *mut c_void,
    buffer: *mut u8,
    size: size_t,
    size_read: *mut size_t,
) -> c_int {
    let parser = data as *mut yaml_parser_t;
    *size_read = libc::fread(buffer as *mut c_void, 1, size, (*parser).input.file);
    (libc::ferror((*parser).input.file) == 0) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn yaml_parser_set_input_string(
    parser: *mut yaml_parser_t,
    input: *const u8,
    size: size_t,
) {
    debug_assert!(!parser.is_null());
    debug_assert!((*parser).read_handler.is_none());
    debug_assert!(!input.is_null());

    (*parser).read_handler = Some(yaml_string_read_handler);
    (*parser).read_handler_data = parser as *mut c_void;

    (*parser).input.string.start = input;
    (*parser).input.string.current = input;
    (*parser).input.string.end = input.add(size);
}

#[no_mangle]
pub unsafe extern "C" fn yaml_parser_set_input_file(
    parser: *mut yaml_parser_t,
    file: *mut FILE,
) {
    debug_assert!(!parser.is_null());
    debug_assert!((*parser).read_handler.is_none());
    debug_assert!(!file.is_null());

    (*parser).read_handler = Some(yaml_file_read_handler);
    (*parser).read_handler_data = parser as *mut c_void;

    (*parser).input.file = file;
}

#[no_mangle]
pub unsafe extern "C" fn yaml_parser_set_input(
    parser: *mut yaml_parser_t,
    handler: yaml_read_handler_t,
    data: *mut c_void,
) {
    debug_assert!(!parser.is_null());
    debug_assert!((*parser).read_handler.is_none());

    (*parser).read_handler = Some(handler);
    (*parser).read_handler_data = data;
}

#[no_mangle]
pub unsafe extern "C" fn yaml_parser_set_encoding(
    parser: *mut yaml_parser_t,
    encoding: yaml_encoding_t,
) {
    debug_assert!(!parser.is_null());
    debug_assert!((*parser).encoding == YAML_ANY_ENCODING);

    (*parser).encoding = encoding;
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_initialize(emitter: *mut yaml_emitter_t) -> c_int {
    debug_assert!(!emitter.is_null());

    libc::memset(emitter as *mut c_void, 0, core::mem::size_of::<yaml_emitter_t>());

    let ok: c_int = 'error: {
        if BUFFER_INIT!(emitter, (*emitter).buffer, OUTPUT_BUFFER_SIZE) == 0 {
            break 'error 0;
        }
        if BUFFER_INIT!(emitter, (*emitter).raw_buffer, OUTPUT_RAW_BUFFER_SIZE) == 0 {
            break 'error 0;
        }
        if STACK_INIT!(emitter, (*emitter).states, yaml_emitter_state_t) == 0 {
            break 'error 0;
        }
        if QUEUE_INIT!(emitter, (*emitter).events, INITIAL_QUEUE_SIZE, yaml_event_t) == 0 {
            break 'error 0;
        }
        if STACK_INIT!(emitter, (*emitter).indents, c_int) == 0 {
            break 'error 0;
        }
        if STACK_INIT!(emitter, (*emitter).tag_directives, yaml_tag_directive_t) == 0 {
            break 'error 0;
        }
        1
    };

    if ok == 0 {
        BUFFER_DEL!(emitter, (*emitter).buffer);
        BUFFER_DEL!(emitter, (*emitter).raw_buffer);
        STACK_DEL!(emitter, (*emitter).states);
        QUEUE_DEL!(emitter, (*emitter).events);
        STACK_DEL!(emitter, (*emitter).indents);
        STACK_DEL!(emitter, (*emitter).tag_directives);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_delete(emitter: *mut yaml_emitter_t) {
    debug_assert!(!emitter.is_null());

    BUFFER_DEL!(emitter, (*emitter).buffer);
    BUFFER_DEL!(emitter, (*emitter).raw_buffer);
    STACK_DEL!(emitter, (*emitter).states);
    while !QUEUE_EMPTY!(emitter, (*emitter).events) {
        let mut ev = DEQUEUE!(emitter, (*emitter).events);
        yaml_event_delete(&mut ev);
    }
    QUEUE_DEL!(emitter, (*emitter).events);
    STACK_DEL!(emitter, (*emitter).indents);
    // Preserve C bug: uses 'empty' instead of 'emitter' as context arg
    let empty = emitter;
    while !STACK_EMPTY!(empty, (*emitter).tag_directives) {
        let tag_directive = POP!(emitter, (*emitter).tag_directives);
        yaml_free(tag_directive.handle as *mut c_void);
        yaml_free(tag_directive.prefix as *mut c_void);
    }
    STACK_DEL!(emitter, (*emitter).tag_directives);
    yaml_free((*emitter).anchors as *mut c_void);

    libc::memset(emitter as *mut c_void, 0, core::mem::size_of::<yaml_emitter_t>());
}

unsafe extern "C" fn yaml_string_write_handler(
    data: *mut c_void,
    buffer: *mut u8,
    size: size_t,
) -> c_int {
    let emitter = data as *mut yaml_emitter_t;

    let size_written = *(*emitter).output.string.size_written;
    let buf_size = (*emitter).output.string.size;

    if buf_size - size_written < size {
        let available = buf_size - size_written;
        libc::memcpy(
            (*emitter).output.string.buffer.add(size_written) as *mut c_void,
            buffer as *const c_void,
            available,
        );
        *(*emitter).output.string.size_written = buf_size;
        return 0;
    }

    libc::memcpy(
        (*emitter).output.string.buffer.add(size_written) as *mut c_void,
        buffer as *const c_void,
        size,
    );
    *(*emitter).output.string.size_written += size;
    1
}

unsafe extern "C" fn yaml_file_write_handler(
    data: *mut c_void,
    buffer: *mut u8,
    size: size_t,
) -> c_int {
    let emitter = data as *mut yaml_emitter_t;
    (libc::fwrite(buffer as *const c_void, 1, size, (*emitter).output.file) == size) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_set_output_string(
    emitter: *mut yaml_emitter_t,
    output: *mut u8,
    size: size_t,
    size_written: *mut size_t,
) {
    debug_assert!(!emitter.is_null());
    debug_assert!((*emitter).write_handler.is_none());
    debug_assert!(!output.is_null());

    (*emitter).write_handler = Some(yaml_string_write_handler);
    (*emitter).write_handler_data = emitter as *mut c_void;

    (*emitter).output.string.buffer = output;
    (*emitter).output.string.size = size;
    (*emitter).output.string.size_written = size_written;
    *size_written = 0;
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_set_output_file(
    emitter: *mut yaml_emitter_t,
    file: *mut FILE,
) {
    debug_assert!(!emitter.is_null());
    debug_assert!((*emitter).write_handler.is_none());
    debug_assert!(!file.is_null());

    (*emitter).write_handler = Some(yaml_file_write_handler);
    (*emitter).write_handler_data = emitter as *mut c_void;

    (*emitter).output.file = file;
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_set_output(
    emitter: *mut yaml_emitter_t,
    handler: yaml_write_handler_t,
    data: *mut c_void,
) {
    debug_assert!(!emitter.is_null());
    debug_assert!((*emitter).write_handler.is_none());

    (*emitter).write_handler = Some(handler);
    (*emitter).write_handler_data = data;
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_set_encoding(
    emitter: *mut yaml_emitter_t,
    encoding: yaml_encoding_t,
) {
    debug_assert!(!emitter.is_null());
    debug_assert!((*emitter).encoding == YAML_ANY_ENCODING);

    (*emitter).encoding = encoding;
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_set_canonical(emitter: *mut yaml_emitter_t, canonical: c_int) {
    debug_assert!(!emitter.is_null());
    (*emitter).canonical = (canonical != 0) as c_int;
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_set_indent(emitter: *mut yaml_emitter_t, indent: c_int) {
    debug_assert!(!emitter.is_null());
    (*emitter).best_indent = if indent > 1 && indent < 10 { indent } else { 2 };
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_set_width(emitter: *mut yaml_emitter_t, width: c_int) {
    debug_assert!(!emitter.is_null());
    (*emitter).best_width = if width >= 0 { width } else { -1 };
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_set_unicode(emitter: *mut yaml_emitter_t, unicode: c_int) {
    debug_assert!(!emitter.is_null());
    (*emitter).unicode = (unicode != 0) as c_int;
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_set_break(
    emitter: *mut yaml_emitter_t,
    line_break: yaml_break_t,
) {
    debug_assert!(!emitter.is_null());
    (*emitter).line_break = line_break;
}

#[no_mangle]
pub unsafe extern "C" fn yaml_token_delete(token: *mut yaml_token_t) {
    debug_assert!(!token.is_null());

    match (*token).type_ {
        YAML_TAG_DIRECTIVE_TOKEN => {
            yaml_free((*token).data.tag_directive.handle as *mut c_void);
            yaml_free((*token).data.tag_directive.prefix as *mut c_void);
        }
        YAML_ALIAS_TOKEN => {
            yaml_free((*token).data.alias.value as *mut c_void);
        }
        YAML_ANCHOR_TOKEN => {
            yaml_free((*token).data.anchor.value as *mut c_void);
        }
        YAML_TAG_TOKEN => {
            yaml_free((*token).data.tag.handle as *mut c_void);
            yaml_free((*token).data.tag.suffix as *mut c_void);
        }
        YAML_SCALAR_TOKEN => {
            yaml_free((*token).data.scalar.value as *mut c_void);
        }
        _ => {}
    }

    libc::memset(token as *mut c_void, 0, core::mem::size_of::<yaml_token_t>());
}

unsafe fn yaml_check_utf8(start: *const yaml_char_t, length: size_t) -> c_int {
    let end = start.add(length);
    let mut pointer = start;

    while pointer < end {
        let octet = *pointer;
        let width: usize = if (octet & 0x80) == 0x00 {
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
        let mut value: u32 = if (octet & 0x80) == 0x00 {
            (octet & 0x7F) as u32
        } else if (octet & 0xE0) == 0xC0 {
            (octet & 0x1F) as u32
        } else if (octet & 0xF0) == 0xE0 {
            (octet & 0x0F) as u32
        } else if (octet & 0xF8) == 0xF0 {
            (octet & 0x07) as u32
        } else {
            0
        };
        if width == 0 {
            return 0;
        }
        if pointer.add(width) > end {
            return 0;
        }
        for k in 1..width {
            let octet = *pointer.add(k);
            if (octet & 0xC0) != 0x80 {
                return 0;
            }
            value = (value << 6) + (octet & 0x3F) as u32;
        }
        if !((width == 1)
            || (width == 2 && value >= 0x80)
            || (width == 3 && value >= 0x800)
            || (width == 4 && value >= 0x10000))
        {
            return 0;
        }
        pointer = pointer.add(width);
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_stream_start_event_initialize(
    event: *mut yaml_event_t,
    encoding: yaml_encoding_t,
) -> c_int {
    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    debug_assert!(!event.is_null());
    STREAM_START_EVENT_INIT!(*event, encoding, mark, mark);
    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_stream_end_event_initialize(event: *mut yaml_event_t) -> c_int {
    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    debug_assert!(!event.is_null());
    STREAM_END_EVENT_INIT!(*event, mark, mark);
    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_document_start_event_initialize(
    event: *mut yaml_event_t,
    version_directive: *mut yaml_version_directive_t,
    tag_directives_start: *mut yaml_tag_directive_t,
    tag_directives_end: *mut yaml_tag_directive_t,
    implicit: c_int,
) -> c_int {
    let mut context = ErrContext { error: YAML_NO_ERROR };
    let context_ptr = &mut context as *mut ErrContext as *mut yaml_parser_t;

    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    let mut version_directive_copy: *mut yaml_version_directive_t = core::ptr::null_mut();
    let mut tag_directives_copy = LocalTagDirStack {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        top: core::ptr::null_mut(),
    };
    let mut value = yaml_tag_directive_t {
        handle: core::ptr::null_mut(),
        prefix: core::ptr::null_mut(),
    };

    debug_assert!(!event.is_null());
    debug_assert!(
        (!tag_directives_start.is_null() && !tag_directives_end.is_null())
            || tag_directives_start == tag_directives_end
    );

    let ok: c_int = 'error: {
        if !version_directive.is_null() {
            version_directive_copy =
                yaml_malloc(core::mem::size_of::<yaml_version_directive_t>())
                    as *mut yaml_version_directive_t;
            if version_directive_copy.is_null() {
                break 'error 0;
            }
            (*version_directive_copy).major = (*version_directive).major;
            (*version_directive_copy).minor = (*version_directive).minor;
        }

        if tag_directives_start != tag_directives_end {
            if STACK_INIT!(context_ptr, tag_directives_copy, yaml_tag_directive_t) == 0 {
                break 'error 0;
            }
            let mut tag_directive = tag_directives_start;
            while tag_directive != tag_directives_end {
                debug_assert!(!(*tag_directive).handle.is_null());
                debug_assert!(!(*tag_directive).prefix.is_null());
                if yaml_check_utf8(
                    (*tag_directive).handle,
                    libc::strlen((*tag_directive).handle as *const c_char),
                ) == 0
                {
                    break 'error 0;
                }
                if yaml_check_utf8(
                    (*tag_directive).prefix,
                    libc::strlen((*tag_directive).prefix as *const c_char),
                ) == 0
                {
                    break 'error 0;
                }
                value.handle = yaml_strdup((*tag_directive).handle);
                value.prefix = yaml_strdup((*tag_directive).prefix);
                if value.handle.is_null() || value.prefix.is_null() {
                    break 'error 0;
                }
                if PUSH!(context_ptr, tag_directives_copy, value) == 0 {
                    break 'error 0;
                }
                value.handle = core::ptr::null_mut();
                value.prefix = core::ptr::null_mut();
                tag_directive = tag_directive.add(1);
            }
        }

        DOCUMENT_START_EVENT_INIT!(
            *event,
            version_directive_copy,
            tag_directives_copy.start,
            tag_directives_copy.top,
            implicit,
            mark,
            mark
        );
        1i32
    };

    if ok == 0 {
        yaml_free(version_directive_copy as *mut c_void);
        while !STACK_EMPTY!(context_ptr, tag_directives_copy) {
            let v = POP!(context_ptr, tag_directives_copy);
            yaml_free(v.handle as *mut c_void);
            yaml_free(v.prefix as *mut c_void);
        }
        STACK_DEL!(context_ptr, tag_directives_copy);
        yaml_free(value.handle as *mut c_void);
        yaml_free(value.prefix as *mut c_void);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_document_end_event_initialize(
    event: *mut yaml_event_t,
    implicit: c_int,
) -> c_int {
    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    debug_assert!(!event.is_null());
    DOCUMENT_END_EVENT_INIT!(*event, implicit, mark, mark);
    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_alias_event_initialize(
    event: *mut yaml_event_t,
    anchor: *const yaml_char_t,
) -> c_int {
    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    debug_assert!(!event.is_null());
    debug_assert!(!anchor.is_null());

    if yaml_check_utf8(anchor, libc::strlen(anchor as *const c_char)) == 0 {
        return 0;
    }

    let anchor_copy = yaml_strdup(anchor);
    if anchor_copy.is_null() {
        return 0;
    }

    ALIAS_EVENT_INIT!(*event, anchor_copy, mark, mark);
    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_scalar_event_initialize(
    event: *mut yaml_event_t,
    anchor: *const yaml_char_t,
    tag: *const yaml_char_t,
    value: *const yaml_char_t,
    length: c_int,
    plain_implicit: c_int,
    quoted_implicit: c_int,
    style: yaml_scalar_style_t,
) -> c_int {
    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    let mut anchor_copy: *mut yaml_char_t = core::ptr::null_mut();
    let mut tag_copy: *mut yaml_char_t = core::ptr::null_mut();
    let mut value_copy: *mut yaml_char_t = core::ptr::null_mut();

    debug_assert!(!event.is_null());
    debug_assert!(!value.is_null());

    let ok: c_int = 'error: {
        if !anchor.is_null() {
            if yaml_check_utf8(anchor, libc::strlen(anchor as *const c_char)) == 0 {
                break 'error 0;
            }
            anchor_copy = yaml_strdup(anchor);
            if anchor_copy.is_null() {
                break 'error 0;
            }
        }

        if !tag.is_null() {
            if yaml_check_utf8(tag, libc::strlen(tag as *const c_char)) == 0 {
                break 'error 0;
            }
            tag_copy = yaml_strdup(tag);
            if tag_copy.is_null() {
                break 'error 0;
            }
        }

        let len: usize = if length < 0 {
            libc::strlen(value as *const c_char)
        } else {
            length as usize
        };

        if yaml_check_utf8(value, len) == 0 {
            break 'error 0;
        }
        value_copy = yaml_malloc(len + 1) as *mut yaml_char_t;
        if value_copy.is_null() {
            break 'error 0;
        }
        libc::memcpy(value_copy as *mut c_void, value as *const c_void, len);
        *value_copy.add(len) = 0;

        SCALAR_EVENT_INIT!(
            *event,
            anchor_copy,
            tag_copy,
            value_copy,
            len,
            plain_implicit,
            quoted_implicit,
            style,
            mark,
            mark
        );
        1i32
    };

    if ok == 0 {
        yaml_free(anchor_copy as *mut c_void);
        yaml_free(tag_copy as *mut c_void);
        yaml_free(value_copy as *mut c_void);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_sequence_start_event_initialize(
    event: *mut yaml_event_t,
    anchor: *const yaml_char_t,
    tag: *const yaml_char_t,
    implicit: c_int,
    style: yaml_sequence_style_t,
) -> c_int {
    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    let mut anchor_copy: *mut yaml_char_t = core::ptr::null_mut();
    let mut tag_copy: *mut yaml_char_t = core::ptr::null_mut();

    debug_assert!(!event.is_null());

    let ok: c_int = 'error: {
        if !anchor.is_null() {
            if yaml_check_utf8(anchor, libc::strlen(anchor as *const c_char)) == 0 {
                break 'error 0;
            }
            anchor_copy = yaml_strdup(anchor);
            if anchor_copy.is_null() {
                break 'error 0;
            }
        }

        if !tag.is_null() {
            if yaml_check_utf8(tag, libc::strlen(tag as *const c_char)) == 0 {
                break 'error 0;
            }
            tag_copy = yaml_strdup(tag);
            if tag_copy.is_null() {
                break 'error 0;
            }
        }

        SEQUENCE_START_EVENT_INIT!(*event, anchor_copy, tag_copy, implicit, style, mark, mark);
        1i32
    };

    if ok == 0 {
        yaml_free(anchor_copy as *mut c_void);
        yaml_free(tag_copy as *mut c_void);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_sequence_end_event_initialize(event: *mut yaml_event_t) -> c_int {
    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    debug_assert!(!event.is_null());
    SEQUENCE_END_EVENT_INIT!(*event, mark, mark);
    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_mapping_start_event_initialize(
    event: *mut yaml_event_t,
    anchor: *const yaml_char_t,
    tag: *const yaml_char_t,
    implicit: c_int,
    style: yaml_mapping_style_t,
) -> c_int {
    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    let mut anchor_copy: *mut yaml_char_t = core::ptr::null_mut();
    let mut tag_copy: *mut yaml_char_t = core::ptr::null_mut();

    debug_assert!(!event.is_null());

    let ok: c_int = 'error: {
        if !anchor.is_null() {
            if yaml_check_utf8(anchor, libc::strlen(anchor as *const c_char)) == 0 {
                break 'error 0;
            }
            anchor_copy = yaml_strdup(anchor);
            if anchor_copy.is_null() {
                break 'error 0;
            }
        }

        if !tag.is_null() {
            if yaml_check_utf8(tag, libc::strlen(tag as *const c_char)) == 0 {
                break 'error 0;
            }
            tag_copy = yaml_strdup(tag);
            if tag_copy.is_null() {
                break 'error 0;
            }
        }

        MAPPING_START_EVENT_INIT!(*event, anchor_copy, tag_copy, implicit, style, mark, mark);
        1i32
    };

    if ok == 0 {
        yaml_free(anchor_copy as *mut c_void);
        yaml_free(tag_copy as *mut c_void);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_mapping_end_event_initialize(event: *mut yaml_event_t) -> c_int {
    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    debug_assert!(!event.is_null());
    MAPPING_END_EVENT_INIT!(*event, mark, mark);
    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_event_delete(event: *mut yaml_event_t) {
    debug_assert!(!event.is_null());

    match (*event).type_ {
        YAML_DOCUMENT_START_EVENT => {
            yaml_free((*event).data.document_start.version_directive as *mut c_void);
            let mut tag_directive = (*event).data.document_start.tag_directives.start;
            while tag_directive != (*event).data.document_start.tag_directives.end {
                yaml_free((*tag_directive).handle as *mut c_void);
                yaml_free((*tag_directive).prefix as *mut c_void);
                tag_directive = tag_directive.add(1);
            }
            yaml_free((*event).data.document_start.tag_directives.start as *mut c_void);
        }
        YAML_ALIAS_EVENT => {
            yaml_free((*event).data.alias.anchor as *mut c_void);
        }
        YAML_SCALAR_EVENT => {
            yaml_free((*event).data.scalar.anchor as *mut c_void);
            yaml_free((*event).data.scalar.tag as *mut c_void);
            yaml_free((*event).data.scalar.value as *mut c_void);
        }
        YAML_SEQUENCE_START_EVENT => {
            yaml_free((*event).data.sequence_start.anchor as *mut c_void);
            yaml_free((*event).data.sequence_start.tag as *mut c_void);
        }
        YAML_MAPPING_START_EVENT => {
            yaml_free((*event).data.mapping_start.anchor as *mut c_void);
            yaml_free((*event).data.mapping_start.tag as *mut c_void);
        }
        _ => {}
    }

    libc::memset(event as *mut c_void, 0, core::mem::size_of::<yaml_event_t>());
}

#[no_mangle]
pub unsafe extern "C" fn yaml_document_initialize(
    document: *mut yaml_document_t,
    version_directive: *mut yaml_version_directive_t,
    tag_directives_start: *mut yaml_tag_directive_t,
    tag_directives_end: *mut yaml_tag_directive_t,
    start_implicit: c_int,
    end_implicit: c_int,
) -> c_int {
    let mut context = ErrContext { error: YAML_NO_ERROR };
    let context_ptr = &mut context as *mut ErrContext as *mut yaml_parser_t;

    let mut nodes = LocalNodeStack {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        top: core::ptr::null_mut(),
    };
    let mut version_directive_copy: *mut yaml_version_directive_t = core::ptr::null_mut();
    let mut tag_directives_copy = LocalTagDirStack {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        top: core::ptr::null_mut(),
    };
    let mut value = yaml_tag_directive_t {
        handle: core::ptr::null_mut(),
        prefix: core::ptr::null_mut(),
    };
    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };

    debug_assert!(!document.is_null());
    debug_assert!(
        (!tag_directives_start.is_null() && !tag_directives_end.is_null())
            || tag_directives_start == tag_directives_end
    );

    let ok: c_int = 'error: {
        if STACK_INIT!(context_ptr, nodes, yaml_node_t) == 0 {
            break 'error 0;
        }

        if !version_directive.is_null() {
            version_directive_copy =
                yaml_malloc(core::mem::size_of::<yaml_version_directive_t>())
                    as *mut yaml_version_directive_t;
            if version_directive_copy.is_null() {
                break 'error 0;
            }
            (*version_directive_copy).major = (*version_directive).major;
            (*version_directive_copy).minor = (*version_directive).minor;
        }

        if tag_directives_start != tag_directives_end {
            if STACK_INIT!(context_ptr, tag_directives_copy, yaml_tag_directive_t) == 0 {
                break 'error 0;
            }
            let mut tag_directive = tag_directives_start;
            while tag_directive != tag_directives_end {
                debug_assert!(!(*tag_directive).handle.is_null());
                debug_assert!(!(*tag_directive).prefix.is_null());
                if yaml_check_utf8(
                    (*tag_directive).handle,
                    libc::strlen((*tag_directive).handle as *const c_char),
                ) == 0
                {
                    break 'error 0;
                }
                if yaml_check_utf8(
                    (*tag_directive).prefix,
                    libc::strlen((*tag_directive).prefix as *const c_char),
                ) == 0
                {
                    break 'error 0;
                }
                value.handle = yaml_strdup((*tag_directive).handle);
                value.prefix = yaml_strdup((*tag_directive).prefix);
                if value.handle.is_null() || value.prefix.is_null() {
                    break 'error 0;
                }
                if PUSH!(context_ptr, tag_directives_copy, value) == 0 {
                    break 'error 0;
                }
                value.handle = core::ptr::null_mut();
                value.prefix = core::ptr::null_mut();
                tag_directive = tag_directive.add(1);
            }
        }

        DOCUMENT_INIT!(
            *document,
            nodes.start,
            nodes.end,
            version_directive_copy,
            tag_directives_copy.start,
            tag_directives_copy.top,
            start_implicit,
            end_implicit,
            mark,
            mark
        );
        1i32
    };

    if ok == 0 {
        STACK_DEL!(context_ptr, nodes);
        yaml_free(version_directive_copy as *mut c_void);
        while !STACK_EMPTY!(context_ptr, tag_directives_copy) {
            let v = POP!(context_ptr, tag_directives_copy);
            yaml_free(v.handle as *mut c_void);
            yaml_free(v.prefix as *mut c_void);
        }
        STACK_DEL!(context_ptr, tag_directives_copy);
        yaml_free(value.handle as *mut c_void);
        yaml_free(value.prefix as *mut c_void);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_document_delete(document: *mut yaml_document_t) {
    // context is referenced but not declared in C - preserved as-is
    // STACK_EMPTY, STACK_DEL, POP don't actually use context arg
    let context: *mut yaml_parser_t = core::ptr::null_mut(); // C code uses undeclared &context
    let _ = context;

    debug_assert!(!document.is_null());

    while !STACK_EMPTY!(context, (*document).nodes) {
        let node = POP!(context, (*document).nodes);
        yaml_free(node.tag as *mut c_void);
        match node.type_ {
            YAML_SCALAR_NODE => {
                yaml_free(node.data.scalar.value as *mut c_void);
            }
            YAML_SEQUENCE_NODE => {
                let mut items = node.data.sequence.items;
                STACK_DEL!(context, items);
            }
            YAML_MAPPING_NODE => {
                let mut pairs = node.data.mapping.pairs;
                STACK_DEL!(context, pairs);
            }
            _ => {
                debug_assert!(false, "Should not happen");
            }
        }
    }
    STACK_DEL!(context, (*document).nodes);

    yaml_free((*document).version_directive as *mut c_void);
    let mut tag_directive = (*document).tag_directives.start;
    while tag_directive != (*document).tag_directives.end {
        yaml_free((*tag_directive).handle as *mut c_void);
        yaml_free((*tag_directive).prefix as *mut c_void);
        tag_directive = tag_directive.add(1);
    }
    yaml_free((*document).tag_directives.start as *mut c_void);

    libc::memset(document as *mut c_void, 0, core::mem::size_of::<yaml_document_t>());
}

#[no_mangle]
pub unsafe extern "C" fn yaml_document_get_node(
    document: *mut yaml_document_t,
    index: c_int,
) -> *mut yaml_node_t {
    debug_assert!(!document.is_null());

    if index > 0
        && (*document).nodes.start.add(index as usize) <= (*document).nodes.top
    {
        return (*document).nodes.start.add(index as usize - 1);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn yaml_document_get_root_node(
    document: *mut yaml_document_t,
) -> *mut yaml_node_t {
    debug_assert!(!document.is_null());

    if (*document).nodes.top != (*document).nodes.start {
        return (*document).nodes.start;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn yaml_document_add_scalar(
    document: *mut yaml_document_t,
    tag: *const yaml_char_t,
    value: *const yaml_char_t,
    length: c_int,
    style: yaml_scalar_style_t,
) -> c_int {
    let mut context = ErrContext { error: YAML_NO_ERROR };
    let context_ptr = &mut context as *mut ErrContext as *mut yaml_parser_t;

    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    let mut tag_copy: *mut yaml_char_t = core::ptr::null_mut();
    let mut value_copy: *mut yaml_char_t = core::ptr::null_mut();
    let mut node = core::mem::zeroed::<yaml_node_t>();

    debug_assert!(!document.is_null());
    debug_assert!(!value.is_null());

    let tag = if tag.is_null() {
        YAML_DEFAULT_SCALAR_TAG.as_ptr() as *const yaml_char_t
    } else {
        tag
    };

    let ok: c_int = 'error: {
        if yaml_check_utf8(tag, libc::strlen(tag as *const c_char)) == 0 {
            break 'error 0;
        }
        tag_copy = yaml_strdup(tag);
        if tag_copy.is_null() {
            break 'error 0;
        }

        let len: usize = if length < 0 {
            libc::strlen(value as *const c_char)
        } else {
            length as usize
        };

        if yaml_check_utf8(value, len) == 0 {
            break 'error 0;
        }
        value_copy = yaml_malloc(len + 1) as *mut yaml_char_t;
        if value_copy.is_null() {
            break 'error 0;
        }
        libc::memcpy(value_copy as *mut c_void, value as *const c_void, len);
        *value_copy.add(len) = 0;

        SCALAR_NODE_INIT!(node, tag_copy, value_copy, len, style, mark, mark);
        if PUSH!(context_ptr, (*document).nodes, node) == 0 {
            break 'error 0;
        }

        ((*document).nodes.top.offset_from((*document).nodes.start)) as c_int
    };

    if ok == 0 {
        yaml_free(tag_copy as *mut c_void);
        yaml_free(value_copy as *mut c_void);
        return 0;
    }

    ok
}

#[no_mangle]
pub unsafe extern "C" fn yaml_document_add_sequence(
    document: *mut yaml_document_t,
    tag: *const yaml_char_t,
    style: yaml_sequence_style_t,
) -> c_int {
    let mut context = ErrContext { error: YAML_NO_ERROR };
    let context_ptr = &mut context as *mut ErrContext as *mut yaml_parser_t;

    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    let mut tag_copy: *mut yaml_char_t = core::ptr::null_mut();
    let mut items = LocalNodeItemStack {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        top: core::ptr::null_mut(),
    };
    let mut node = core::mem::zeroed::<yaml_node_t>();

    debug_assert!(!document.is_null());

    let tag = if tag.is_null() {
        YAML_DEFAULT_SEQUENCE_TAG.as_ptr() as *const yaml_char_t
    } else {
        tag
    };

    let ok: c_int = 'error: {
        if yaml_check_utf8(tag, libc::strlen(tag as *const c_char)) == 0 {
            break 'error 0;
        }
        tag_copy = yaml_strdup(tag);
        if tag_copy.is_null() {
            break 'error 0;
        }

        if STACK_INIT!(context_ptr, items, yaml_node_item_t) == 0 {
            break 'error 0;
        }

        SEQUENCE_NODE_INIT!(node, tag_copy, items.start, items.end, style, mark, mark);
        if PUSH!(context_ptr, (*document).nodes, node) == 0 {
            break 'error 0;
        }

        ((*document).nodes.top.offset_from((*document).nodes.start)) as c_int
    };

    if ok == 0 {
        STACK_DEL!(context_ptr, items);
        yaml_free(tag_copy as *mut c_void);
        return 0;
    }

    ok
}

#[no_mangle]
pub unsafe extern "C" fn yaml_document_add_mapping(
    document: *mut yaml_document_t,
    tag: *const yaml_char_t,
    style: yaml_mapping_style_t,
) -> c_int {
    let mut context = ErrContext { error: YAML_NO_ERROR };
    let context_ptr = &mut context as *mut ErrContext as *mut yaml_parser_t;

    let mark = yaml_mark_t { index: 0, line: 0, column: 0 };
    let mut tag_copy: *mut yaml_char_t = core::ptr::null_mut();
    let mut pairs = LocalNodePairStack {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        top: core::ptr::null_mut(),
    };
    let mut node = core::mem::zeroed::<yaml_node_t>();

    debug_assert!(!document.is_null());

    let tag = if tag.is_null() {
        YAML_DEFAULT_MAPPING_TAG.as_ptr() as *const yaml_char_t
    } else {
        tag
    };

    let ok: c_int = 'error: {
        if yaml_check_utf8(tag, libc::strlen(tag as *const c_char)) == 0 {
            break 'error 0;
        }
        tag_copy = yaml_strdup(tag);
        if tag_copy.is_null() {
            break 'error 0;
        }

        if STACK_INIT!(context_ptr, pairs, yaml_node_pair_t) == 0 {
            break 'error 0;
        }

        MAPPING_NODE_INIT!(node, tag_copy, pairs.start, pairs.end, style, mark, mark);
        if PUSH!(context_ptr, (*document).nodes, node) == 0 {
            break 'error 0;
        }

        ((*document).nodes.top.offset_from((*document).nodes.start)) as c_int
    };

    if ok == 0 {
        STACK_DEL!(context_ptr, pairs);
        yaml_free(tag_copy as *mut c_void);
        return 0;
    }

    ok
}

#[no_mangle]
pub unsafe extern "C" fn yaml_document_append_sequence_item(
    document: *mut yaml_document_t,
    sequence: c_int,
    item: c_int,
) -> c_int {
    let mut context = ErrContext { error: YAML_NO_ERROR };
    let context_ptr = &mut context as *mut ErrContext as *mut yaml_parser_t;

    debug_assert!(!document.is_null());
    debug_assert!(
        sequence > 0
            && (*document).nodes.start.add(sequence as usize) <= (*document).nodes.top
    );
    debug_assert!(
        (*(*document).nodes.start.add(sequence as usize - 1)).type_ == YAML_SEQUENCE_NODE
    );
    debug_assert!(
        item > 0 && (*document).nodes.start.add(item as usize) <= (*document).nodes.top
    );

    if PUSH!(
        context_ptr,
        (*(*document).nodes.start.add(sequence as usize - 1))
            .data
            .sequence
            .items,
        item
    ) == 0
    {
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_document_append_mapping_pair(
    document: *mut yaml_document_t,
    mapping: c_int,
    key: c_int,
    value: c_int,
) -> c_int {
    let mut context = ErrContext { error: YAML_NO_ERROR };
    let context_ptr = &mut context as *mut ErrContext as *mut yaml_parser_t;

    debug_assert!(!document.is_null());
    debug_assert!(
        mapping > 0
            && (*document).nodes.start.add(mapping as usize) <= (*document).nodes.top
    );
    debug_assert!(
        (*(*document).nodes.start.add(mapping as usize - 1)).type_ == YAML_MAPPING_NODE
    );
    debug_assert!(
        key > 0 && (*document).nodes.start.add(key as usize) <= (*document).nodes.top
    );
    debug_assert!(
        value > 0 && (*document).nodes.start.add(value as usize) <= (*document).nodes.top
    );

    let pair = yaml_node_pair_t { key, value };

    if PUSH!(
        context_ptr,
        (*(*document).nodes.start.add(mapping as usize - 1))
            .data
            .mapping
            .pairs,
        pair
    ) == 0
    {
        return 0;
    }

    1
}
