#![allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    unused_variables,
    unused_assignments,
    unused_mut,
    dead_code,
    improper_ctypes_definitions,
    clippy::all
)]

pub mod api;
pub mod reader;
pub mod writer;
pub mod scanner;
pub mod parser;
pub mod loader;
pub mod dumper;
pub mod emitter;

use libc::{c_char, c_int, c_void, size_t, FILE};

// ---- Version ----

pub const YAML_VERSION_MAJOR: c_int = 0;
pub const YAML_VERSION_MINOR: c_int = 2;
pub const YAML_VERSION_PATCH: c_int = 5;
pub const YAML_VERSION_STRING: &[u8] = b"0.2.5\0";

// ---- Buffer size constants ----

pub const INPUT_RAW_BUFFER_SIZE: usize = 16384;
pub const INPUT_BUFFER_SIZE: usize = INPUT_RAW_BUFFER_SIZE * 3;
pub const OUTPUT_BUFFER_SIZE: usize = 16384;
pub const OUTPUT_RAW_BUFFER_SIZE: usize = OUTPUT_BUFFER_SIZE * 2 + 2;
pub const MAX_FILE_SIZE: usize = !0usize / 2;
pub const INITIAL_STACK_SIZE: usize = 16;
pub const INITIAL_QUEUE_SIZE: usize = 16;
pub const INITIAL_STRING_SIZE: usize = 16;

// ---- Public type aliases ----

pub type yaml_char_t = u8;

// ---- Enums ----

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum yaml_encoding_t {
    YAML_ANY_ENCODING = 0,
    YAML_UTF8_ENCODING = 1,
    YAML_UTF16LE_ENCODING = 2,
    YAML_UTF16BE_ENCODING = 3,
}
pub use yaml_encoding_t::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum yaml_break_t {
    YAML_ANY_BREAK = 0,
    YAML_CR_BREAK = 1,
    YAML_LN_BREAK = 2,
    YAML_CRLN_BREAK = 3,
}
pub use yaml_break_t::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum yaml_error_type_t {
    YAML_NO_ERROR = 0,
    YAML_MEMORY_ERROR = 1,
    YAML_READER_ERROR = 2,
    YAML_SCANNER_ERROR = 3,
    YAML_PARSER_ERROR = 4,
    YAML_COMPOSER_ERROR = 5,
    YAML_WRITER_ERROR = 6,
    YAML_EMITTER_ERROR = 7,
}
pub use yaml_error_type_t::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum yaml_scalar_style_t {
    YAML_ANY_SCALAR_STYLE = 0,
    YAML_PLAIN_SCALAR_STYLE = 1,
    YAML_SINGLE_QUOTED_SCALAR_STYLE = 2,
    YAML_DOUBLE_QUOTED_SCALAR_STYLE = 3,
    YAML_LITERAL_SCALAR_STYLE = 4,
    YAML_FOLDED_SCALAR_STYLE = 5,
}
pub use yaml_scalar_style_t::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum yaml_sequence_style_t {
    YAML_ANY_SEQUENCE_STYLE = 0,
    YAML_BLOCK_SEQUENCE_STYLE = 1,
    YAML_FLOW_SEQUENCE_STYLE = 2,
}
pub use yaml_sequence_style_t::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum yaml_mapping_style_t {
    YAML_ANY_MAPPING_STYLE = 0,
    YAML_BLOCK_MAPPING_STYLE = 1,
    YAML_FLOW_MAPPING_STYLE = 2,
}
pub use yaml_mapping_style_t::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum yaml_token_type_t {
    YAML_NO_TOKEN = 0,
    YAML_STREAM_START_TOKEN = 1,
    YAML_STREAM_END_TOKEN = 2,
    YAML_VERSION_DIRECTIVE_TOKEN = 3,
    YAML_TAG_DIRECTIVE_TOKEN = 4,
    YAML_DOCUMENT_START_TOKEN = 5,
    YAML_DOCUMENT_END_TOKEN = 6,
    YAML_BLOCK_SEQUENCE_START_TOKEN = 7,
    YAML_BLOCK_MAPPING_START_TOKEN = 8,
    YAML_BLOCK_END_TOKEN = 9,
    YAML_FLOW_SEQUENCE_START_TOKEN = 10,
    YAML_FLOW_SEQUENCE_END_TOKEN = 11,
    YAML_FLOW_MAPPING_START_TOKEN = 12,
    YAML_FLOW_MAPPING_END_TOKEN = 13,
    YAML_BLOCK_ENTRY_TOKEN = 14,
    YAML_FLOW_ENTRY_TOKEN = 15,
    YAML_KEY_TOKEN = 16,
    YAML_VALUE_TOKEN = 17,
    YAML_ALIAS_TOKEN = 18,
    YAML_ANCHOR_TOKEN = 19,
    YAML_TAG_TOKEN = 20,
    YAML_SCALAR_TOKEN = 21,
}
pub use yaml_token_type_t::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum yaml_event_type_t {
    YAML_NO_EVENT = 0,
    YAML_STREAM_START_EVENT = 1,
    YAML_STREAM_END_EVENT = 2,
    YAML_DOCUMENT_START_EVENT = 3,
    YAML_DOCUMENT_END_EVENT = 4,
    YAML_ALIAS_EVENT = 5,
    YAML_SCALAR_EVENT = 6,
    YAML_SEQUENCE_START_EVENT = 7,
    YAML_SEQUENCE_END_EVENT = 8,
    YAML_MAPPING_START_EVENT = 9,
    YAML_MAPPING_END_EVENT = 10,
}
pub use yaml_event_type_t::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum yaml_node_type_t {
    YAML_NO_NODE = 0,
    YAML_SCALAR_NODE = 1,
    YAML_SEQUENCE_NODE = 2,
    YAML_MAPPING_NODE = 3,
}
pub use yaml_node_type_t::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum yaml_parser_state_t {
    YAML_PARSE_STREAM_START_STATE = 0,
    YAML_PARSE_IMPLICIT_DOCUMENT_START_STATE = 1,
    YAML_PARSE_DOCUMENT_START_STATE = 2,
    YAML_PARSE_DOCUMENT_CONTENT_STATE = 3,
    YAML_PARSE_DOCUMENT_END_STATE = 4,
    YAML_PARSE_BLOCK_NODE_STATE = 5,
    YAML_PARSE_BLOCK_NODE_OR_INDENTLESS_SEQUENCE_STATE = 6,
    YAML_PARSE_FLOW_NODE_STATE = 7,
    YAML_PARSE_BLOCK_SEQUENCE_FIRST_ENTRY_STATE = 8,
    YAML_PARSE_BLOCK_SEQUENCE_ENTRY_STATE = 9,
    YAML_PARSE_INDENTLESS_SEQUENCE_ENTRY_STATE = 10,
    YAML_PARSE_BLOCK_MAPPING_FIRST_KEY_STATE = 11,
    YAML_PARSE_BLOCK_MAPPING_KEY_STATE = 12,
    YAML_PARSE_BLOCK_MAPPING_VALUE_STATE = 13,
    YAML_PARSE_FLOW_SEQUENCE_FIRST_ENTRY_STATE = 14,
    YAML_PARSE_FLOW_SEQUENCE_ENTRY_STATE = 15,
    YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_KEY_STATE = 16,
    YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_VALUE_STATE = 17,
    YAML_PARSE_FLOW_SEQUENCE_ENTRY_MAPPING_END_STATE = 18,
    YAML_PARSE_FLOW_MAPPING_FIRST_KEY_STATE = 19,
    YAML_PARSE_FLOW_MAPPING_KEY_STATE = 20,
    YAML_PARSE_FLOW_MAPPING_VALUE_STATE = 21,
    YAML_PARSE_FLOW_MAPPING_EMPTY_VALUE_STATE = 22,
    YAML_PARSE_END_STATE = 23,
}
pub use yaml_parser_state_t::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum yaml_emitter_state_t {
    YAML_EMIT_STREAM_START_STATE = 0,
    YAML_EMIT_FIRST_DOCUMENT_START_STATE = 1,
    YAML_EMIT_DOCUMENT_START_STATE = 2,
    YAML_EMIT_DOCUMENT_CONTENT_STATE = 3,
    YAML_EMIT_DOCUMENT_END_STATE = 4,
    YAML_EMIT_FLOW_SEQUENCE_FIRST_ITEM_STATE = 5,
    YAML_EMIT_FLOW_SEQUENCE_ITEM_STATE = 6,
    YAML_EMIT_FLOW_MAPPING_FIRST_KEY_STATE = 7,
    YAML_EMIT_FLOW_MAPPING_KEY_STATE = 8,
    YAML_EMIT_FLOW_MAPPING_SIMPLE_VALUE_STATE = 9,
    YAML_EMIT_FLOW_MAPPING_VALUE_STATE = 10,
    YAML_EMIT_BLOCK_SEQUENCE_FIRST_ITEM_STATE = 11,
    YAML_EMIT_BLOCK_SEQUENCE_ITEM_STATE = 12,
    YAML_EMIT_BLOCK_MAPPING_FIRST_KEY_STATE = 13,
    YAML_EMIT_BLOCK_MAPPING_KEY_STATE = 14,
    YAML_EMIT_BLOCK_MAPPING_SIMPLE_VALUE_STATE = 15,
    YAML_EMIT_BLOCK_MAPPING_VALUE_STATE = 16,
    YAML_EMIT_END_STATE = 17,
}
pub use yaml_emitter_state_t::*;

// ---- Structs ----

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct yaml_mark_t {
    pub index: size_t,
    pub line: size_t,
    pub column: size_t,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct yaml_version_directive_t {
    pub major: c_int,
    pub minor: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct yaml_tag_directive_t {
    pub handle: *mut yaml_char_t,
    pub prefix: *mut yaml_char_t,
}

// ---- Token union helpers ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_token_stream_start_t {
    pub encoding: yaml_encoding_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_token_alias_t {
    pub value: *mut yaml_char_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_token_anchor_t {
    pub value: *mut yaml_char_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_token_tag_t {
    pub handle: *mut yaml_char_t,
    pub suffix: *mut yaml_char_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_token_scalar_t {
    pub value: *mut yaml_char_t,
    pub length: size_t,
    pub style: yaml_scalar_style_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_token_version_directive_t {
    pub major: c_int,
    pub minor: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_token_tag_directive_t {
    pub handle: *mut yaml_char_t,
    pub prefix: *mut yaml_char_t,
}

#[repr(C)]
pub union yaml_token_data_t {
    pub stream_start: yaml_token_stream_start_t,
    pub alias: yaml_token_alias_t,
    pub anchor: yaml_token_anchor_t,
    pub tag: yaml_token_tag_t,
    pub scalar: yaml_token_scalar_t,
    pub version_directive: yaml_token_version_directive_t,
    pub tag_directive: yaml_token_tag_directive_t,
}

impl Copy for yaml_token_data_t {}
impl Clone for yaml_token_data_t {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_token_t {
    pub type_: yaml_token_type_t,
    pub data: yaml_token_data_t,
    pub start_mark: yaml_mark_t,
    pub end_mark: yaml_mark_t,
}

// ---- Event union helpers ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_event_stream_start_t {
    pub encoding: yaml_encoding_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_event_document_start_tag_directives_t {
    pub start: *mut yaml_tag_directive_t,
    pub end: *mut yaml_tag_directive_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_event_document_start_t {
    pub version_directive: *mut yaml_version_directive_t,
    pub tag_directives: yaml_event_document_start_tag_directives_t,
    pub implicit: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_event_document_end_t {
    pub implicit: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_event_alias_t {
    pub anchor: *mut yaml_char_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_event_scalar_t {
    pub anchor: *mut yaml_char_t,
    pub tag: *mut yaml_char_t,
    pub value: *mut yaml_char_t,
    pub length: size_t,
    pub plain_implicit: c_int,
    pub quoted_implicit: c_int,
    pub style: yaml_scalar_style_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_event_sequence_start_t {
    pub anchor: *mut yaml_char_t,
    pub tag: *mut yaml_char_t,
    pub implicit: c_int,
    pub style: yaml_sequence_style_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_event_mapping_start_t {
    pub anchor: *mut yaml_char_t,
    pub tag: *mut yaml_char_t,
    pub implicit: c_int,
    pub style: yaml_mapping_style_t,
}

#[repr(C)]
pub union yaml_event_data_t {
    pub stream_start: yaml_event_stream_start_t,
    pub document_start: yaml_event_document_start_t,
    pub document_end: yaml_event_document_end_t,
    pub alias: yaml_event_alias_t,
    pub scalar: yaml_event_scalar_t,
    pub sequence_start: yaml_event_sequence_start_t,
    pub mapping_start: yaml_event_mapping_start_t,
}

impl Copy for yaml_event_data_t {}
impl Clone for yaml_event_data_t {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_event_t {
    pub type_: yaml_event_type_t,
    pub data: yaml_event_data_t,
    pub start_mark: yaml_mark_t,
    pub end_mark: yaml_mark_t,
}

// ---- Node types ----

pub type yaml_node_item_t = c_int;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_node_pair_t {
    pub key: c_int,
    pub value: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_node_scalar_t {
    pub value: *mut yaml_char_t,
    pub length: size_t,
    pub style: yaml_scalar_style_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_node_item_stack_t {
    pub start: *mut yaml_node_item_t,
    pub end: *mut yaml_node_item_t,
    pub top: *mut yaml_node_item_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_node_sequence_t {
    pub items: yaml_node_item_stack_t,
    pub style: yaml_sequence_style_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_node_pair_stack_t {
    pub start: *mut yaml_node_pair_t,
    pub end: *mut yaml_node_pair_t,
    pub top: *mut yaml_node_pair_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_node_mapping_t {
    pub pairs: yaml_node_pair_stack_t,
    pub style: yaml_mapping_style_t,
}

#[repr(C)]
pub union yaml_node_data_t {
    pub scalar: yaml_node_scalar_t,
    pub sequence: yaml_node_sequence_t,
    pub mapping: yaml_node_mapping_t,
}

impl Copy for yaml_node_data_t {}
impl Clone for yaml_node_data_t {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_node_t {
    pub type_: yaml_node_type_t,
    pub tag: *mut yaml_char_t,
    pub data: yaml_node_data_t,
    pub start_mark: yaml_mark_t,
    pub end_mark: yaml_mark_t,
}

// ---- Document ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_node_stack_t {
    pub start: *mut yaml_node_t,
    pub end: *mut yaml_node_t,
    pub top: *mut yaml_node_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_tag_directive_range_t {
    pub start: *mut yaml_tag_directive_t,
    pub end: *mut yaml_tag_directive_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_document_t {
    pub nodes: yaml_node_stack_t,
    pub version_directive: *mut yaml_version_directive_t,
    pub tag_directives: yaml_tag_directive_range_t,
    pub start_implicit: c_int,
    pub end_implicit: c_int,
    pub start_mark: yaml_mark_t,
    pub end_mark: yaml_mark_t,
}

// ---- Handler types ----

pub type yaml_read_handler_t =
    unsafe extern "C" fn(data: *mut c_void, buffer: *mut u8, size: size_t, size_read: *mut size_t) -> c_int;

pub type yaml_write_handler_t =
    unsafe extern "C" fn(data: *mut c_void, buffer: *mut u8, size: size_t) -> c_int;

// ---- Simple key ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_simple_key_t {
    pub possible: c_int,
    pub required: c_int,
    pub token_number: size_t,
    pub mark: yaml_mark_t,
}

// ---- Alias data ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_alias_data_t {
    pub anchor: *mut yaml_char_t,
    pub index: c_int,
    pub mark: yaml_mark_t,
}

// ---- Anchors ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_anchors_t {
    pub references: c_int,
    pub anchor: c_int,
    pub serialized: c_int,
}

// ---- Internal buffer/string types ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_string_t {
    pub start: *mut yaml_char_t,
    pub end: *mut yaml_char_t,
    pub pointer: *mut yaml_char_t,
}

// ---- Parser input union ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_parser_input_string_t {
    pub start: *const u8,
    pub end: *const u8,
    pub current: *const u8,
}

#[repr(C)]
pub union yaml_parser_input_t {
    pub string: yaml_parser_input_string_t,
    pub file: *mut FILE,
}

impl Copy for yaml_parser_input_t {}
impl Clone for yaml_parser_input_t {
    fn clone(&self) -> Self {
        *self
    }
}

// ---- Parser buffer ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_parser_buffer_t {
    pub start: *mut yaml_char_t,
    pub end: *mut yaml_char_t,
    pub pointer: *mut yaml_char_t,
    pub last: *mut yaml_char_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_raw_buffer_t {
    pub start: *mut u8,
    pub end: *mut u8,
    pub pointer: *mut u8,
    pub last: *mut u8,
}

// ---- Token queue ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_token_queue_t {
    pub start: *mut yaml_token_t,
    pub end: *mut yaml_token_t,
    pub head: *mut yaml_token_t,
    pub tail: *mut yaml_token_t,
}

// ---- Stack types ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_int_stack_t {
    pub start: *mut c_int,
    pub end: *mut c_int,
    pub top: *mut c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_simple_key_stack_t {
    pub start: *mut yaml_simple_key_t,
    pub end: *mut yaml_simple_key_t,
    pub top: *mut yaml_simple_key_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_parser_state_stack_t {
    pub start: *mut yaml_parser_state_t,
    pub end: *mut yaml_parser_state_t,
    pub top: *mut yaml_parser_state_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_mark_stack_t {
    pub start: *mut yaml_mark_t,
    pub end: *mut yaml_mark_t,
    pub top: *mut yaml_mark_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_tag_directive_stack_t {
    pub start: *mut yaml_tag_directive_t,
    pub end: *mut yaml_tag_directive_t,
    pub top: *mut yaml_tag_directive_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_alias_data_stack_t {
    pub start: *mut yaml_alias_data_t,
    pub end: *mut yaml_alias_data_t,
    pub top: *mut yaml_alias_data_t,
}

// ---- Parser ----

#[repr(C)]
pub struct yaml_parser_t {
    // Error handling
    pub error: yaml_error_type_t,
    pub problem: *const c_char,
    pub problem_offset: size_t,
    pub problem_value: c_int,
    pub problem_mark: yaml_mark_t,
    pub context: *const c_char,
    pub context_mark: yaml_mark_t,
    // Reader
    pub read_handler: Option<yaml_read_handler_t>,
    pub read_handler_data: *mut c_void,
    pub input: yaml_parser_input_t,
    pub eof: c_int,
    pub buffer: yaml_parser_buffer_t,
    pub unread: size_t,
    pub raw_buffer: yaml_raw_buffer_t,
    pub encoding: yaml_encoding_t,
    pub offset: size_t,
    pub mark: yaml_mark_t,
    // Scanner
    pub stream_start_produced: c_int,
    pub stream_end_produced: c_int,
    pub flow_level: c_int,
    pub tokens: yaml_token_queue_t,
    pub tokens_parsed: size_t,
    pub token_available: c_int,
    pub indents: yaml_int_stack_t,
    pub indent: c_int,
    pub simple_key_allowed: c_int,
    pub simple_keys: yaml_simple_key_stack_t,
    // Parser
    pub states: yaml_parser_state_stack_t,
    pub state: yaml_parser_state_t,
    pub marks: yaml_mark_stack_t,
    pub tag_directives: yaml_tag_directive_stack_t,
    // Loader
    pub aliases: yaml_alias_data_stack_t,
    pub document: *mut yaml_document_t,
}

// ---- Emitter output union ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_emitter_output_string_t {
    pub buffer: *mut u8,
    pub size: size_t,
    pub size_written: *mut size_t,
}

#[repr(C)]
pub union yaml_emitter_output_t {
    pub string: yaml_emitter_output_string_t,
    pub file: *mut FILE,
}

impl Copy for yaml_emitter_output_t {}
impl Clone for yaml_emitter_output_t {
    fn clone(&self) -> Self {
        *self
    }
}

// ---- Event queue ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_event_queue_t {
    pub start: *mut yaml_event_t,
    pub end: *mut yaml_event_t,
    pub head: *mut yaml_event_t,
    pub tail: *mut yaml_event_t,
}

// ---- Emitter state stack ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_emitter_state_stack_t {
    pub start: *mut yaml_emitter_state_t,
    pub end: *mut yaml_emitter_state_t,
    pub top: *mut yaml_emitter_state_t,
}

// ---- Emitter analysis structs ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_emitter_anchor_data_t {
    pub anchor: *mut yaml_char_t,
    pub anchor_length: size_t,
    pub alias: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_emitter_tag_data_t {
    pub handle: *mut yaml_char_t,
    pub handle_length: size_t,
    pub suffix: *mut yaml_char_t,
    pub suffix_length: size_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct yaml_emitter_scalar_data_t {
    pub value: *mut yaml_char_t,
    pub length: size_t,
    pub multiline: c_int,
    pub flow_plain_allowed: c_int,
    pub block_plain_allowed: c_int,
    pub single_quoted_allowed: c_int,
    pub block_allowed: c_int,
    pub style: yaml_scalar_style_t,
}

// ---- Emitter ----

#[repr(C)]
pub struct yaml_emitter_t {
    // Error handling
    pub error: yaml_error_type_t,
    pub problem: *const c_char,
    // Writer
    pub write_handler: Option<yaml_write_handler_t>,
    pub write_handler_data: *mut c_void,
    pub output: yaml_emitter_output_t,
    pub buffer: yaml_parser_buffer_t,
    pub raw_buffer: yaml_raw_buffer_t,
    pub encoding: yaml_encoding_t,
    // Emitter
    pub canonical: c_int,
    pub best_indent: c_int,
    pub best_width: c_int,
    pub unicode: c_int,
    pub line_break: yaml_break_t,
    pub states: yaml_emitter_state_stack_t,
    pub state: yaml_emitter_state_t,
    pub events: yaml_event_queue_t,
    pub indents: yaml_int_stack_t,
    pub tag_directives: yaml_tag_directive_stack_t,
    pub indent: c_int,
    pub flow_level: c_int,
    pub root_context: c_int,
    pub sequence_context: c_int,
    pub mapping_context: c_int,
    pub simple_key_context: c_int,
    pub line: c_int,
    pub column: c_int,
    pub whitespace: c_int,
    pub indention: c_int,
    pub open_ended: c_int,
    pub anchor_data: yaml_emitter_anchor_data_t,
    pub tag_data: yaml_emitter_tag_data_t,
    pub scalar_data: yaml_emitter_scalar_data_t,
    // Dumper
    pub opened: c_int,
    pub closed: c_int,
    pub anchors: *mut yaml_anchors_t,
    pub last_anchor_id: c_int,
    pub document: *mut yaml_document_t,
}

// ============================================================
// Macros
// ============================================================

#[macro_export]
macro_rules! BUFFER_INIT {
    ($context:expr, $buffer:expr, $size:expr) => {{
        let sz = $size as usize;
        let ptr = $crate::api::yaml_malloc(sz) as *mut $crate::yaml_char_t;
        if !ptr.is_null() {
            $buffer.start = ptr;
            $buffer.last = ptr;
            $buffer.pointer = ptr;
            $buffer.end = ptr.add(sz);
            1i32
        } else {
            (*$context).error = $crate::YAML_MEMORY_ERROR;
            0i32
        }
    }};
}

#[macro_export]
macro_rules! BUFFER_DEL {
    ($context:expr, $buffer:expr) => {{
        $crate::api::yaml_free($buffer.start as *mut libc::c_void);
        $buffer.start = core::ptr::null_mut();
        $buffer.pointer = core::ptr::null_mut();
        $buffer.end = core::ptr::null_mut();
    }};
}

#[macro_export]
macro_rules! STRING_ASSIGN {
    ($value:expr, $string:expr, $length:expr) => {{
        $value.start = $string;
        $value.end = $string.add($length as usize);
        $value.pointer = $string;
    }};
}

#[macro_export]
macro_rules! STRING_INIT {
    ($context:expr, $string:expr, $size:expr) => {{
        let sz = $size as usize;
        let ptr = $crate::api::yaml_malloc(sz) as *mut $crate::yaml_char_t;
        if !ptr.is_null() {
            $string.start = ptr;
            $string.pointer = ptr;
            $string.end = ptr.add(sz);
            libc::memset(ptr as *mut libc::c_void, 0, sz);
            1i32
        } else {
            (*$context).error = $crate::YAML_MEMORY_ERROR;
            0i32
        }
    }};
}

#[macro_export]
macro_rules! STRING_DEL {
    ($context:expr, $string:expr) => {{
        $crate::api::yaml_free($string.start as *mut libc::c_void);
        $string.start = core::ptr::null_mut();
        $string.pointer = core::ptr::null_mut();
        $string.end = core::ptr::null_mut();
    }};
}

#[macro_export]
macro_rules! STRING_EXTEND {
    ($context:expr, $string:expr) => {{
        if $string.pointer.add(5) < $string.end {
            1i32
        } else {
            $crate::api::yaml_string_extend(
                &mut $string.start,
                &mut $string.pointer,
                &mut $string.end,
            )
        }
    }};
}

#[macro_export]
macro_rules! CLEAR {
    ($context:expr, $string:expr) => {{
        $string.pointer = $string.start;
        libc::memset(
            $string.start as *mut libc::c_void,
            0,
            $string.end.offset_from($string.start) as usize,
        );
    }};
}

#[macro_export]
macro_rules! JOIN {
    ($context:expr, $string_a:expr, $string_b:expr) => {{
        let r = $crate::api::yaml_string_join(
            &mut $string_a.start,
            &mut $string_a.pointer,
            &mut $string_a.end,
            &mut $string_b.start,
            &mut $string_b.pointer,
            &mut $string_b.end,
        );
        if r != 0 {
            $string_b.pointer = $string_b.start;
            1i32
        } else {
            (*$context).error = $crate::YAML_MEMORY_ERROR;
            0i32
        }
    }};
}

// Character check macros

#[macro_export]
macro_rules! CHECK_AT {
    ($string:expr, $octet:expr, $offset:expr) => {
        *$string.pointer.add($offset as usize) == $octet as u8
    };
}

#[macro_export]
macro_rules! CHECK {
    ($string:expr, $octet:expr) => {
        $crate::CHECK_AT!($string, $octet, 0)
    };
}

#[macro_export]
macro_rules! IS_ALPHA_AT {
    ($string:expr, $offset:expr) => {{
        let c = *$string.pointer.add($offset as usize);
        (c >= b'0' && c <= b'9')
            || (c >= b'A' && c <= b'Z')
            || (c >= b'a' && c <= b'z')
            || c == b'_'
            || c == b'-'
    }};
}

#[macro_export]
macro_rules! IS_ALPHA {
    ($string:expr) => {
        $crate::IS_ALPHA_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_DIGIT_AT {
    ($string:expr, $offset:expr) => {{
        let c = *$string.pointer.add($offset as usize);
        c >= b'0' && c <= b'9'
    }};
}

#[macro_export]
macro_rules! IS_DIGIT {
    ($string:expr) => {
        $crate::IS_DIGIT_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! AS_DIGIT_AT {
    ($string:expr, $offset:expr) => {
        (*$string.pointer.add($offset as usize) - b'0') as libc::c_int
    };
}

#[macro_export]
macro_rules! AS_DIGIT {
    ($string:expr) => {
        $crate::AS_DIGIT_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_HEX_AT {
    ($string:expr, $offset:expr) => {{
        let c = *$string.pointer.add($offset as usize);
        (c >= b'0' && c <= b'9') || (c >= b'A' && c <= b'F') || (c >= b'a' && c <= b'f')
    }};
}

#[macro_export]
macro_rules! IS_HEX {
    ($string:expr) => {
        $crate::IS_HEX_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! AS_HEX_AT {
    ($string:expr, $offset:expr) => {{
        let c = *$string.pointer.add($offset as usize);
        if c >= b'A' && c <= b'F' {
            (c - b'A' + 10) as libc::c_int
        } else if c >= b'a' && c <= b'f' {
            (c - b'a' + 10) as libc::c_int
        } else {
            (c - b'0') as libc::c_int
        }
    }};
}

#[macro_export]
macro_rules! AS_HEX {
    ($string:expr) => {
        $crate::AS_HEX_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_ASCII_AT {
    ($string:expr, $offset:expr) => {
        *$string.pointer.add($offset as usize) <= 0x7Fu8
    };
}

#[macro_export]
macro_rules! IS_ASCII {
    ($string:expr) => {
        $crate::IS_ASCII_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_PRINTABLE_AT {
    ($string:expr, $offset:expr) => {{
        let p = $string.pointer.add($offset as usize);
        let c = *p;
        (c == 0x0Au8)
            || (c >= 0x20u8 && c <= 0x7Eu8)
            || (c == 0xC2u8 && *p.add(1) >= 0xA0u8)
            || (c > 0xC2u8 && c < 0xEDu8)
            || (c == 0xEDu8 && *p.add(1) < 0xA0u8)
            || (c == 0xEEu8)
            || (c == 0xEFu8
                && !(*p.add(1) == 0xBBu8 && *p.add(2) == 0xBFu8)
                && !(*p.add(1) == 0xBFu8 && (*p.add(2) == 0xBEu8 || *p.add(2) == 0xBFu8)))
    }};
}

#[macro_export]
macro_rules! IS_PRINTABLE {
    ($string:expr) => {
        $crate::IS_PRINTABLE_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_Z_AT {
    ($string:expr, $offset:expr) => {
        *$string.pointer.add($offset as usize) == 0u8
    };
}

#[macro_export]
macro_rules! IS_Z {
    ($string:expr) => {
        $crate::IS_Z_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_BOM_AT {
    ($string:expr, $offset:expr) => {
        *$string.pointer.add($offset as usize) == 0xEFu8
            && *$string.pointer.add($offset as usize + 1) == 0xBBu8
            && *$string.pointer.add($offset as usize + 2) == 0xBFu8
    };
}

#[macro_export]
macro_rules! IS_BOM {
    ($string:expr) => {
        $crate::IS_BOM_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_SPACE_AT {
    ($string:expr, $offset:expr) => {
        *$string.pointer.add($offset as usize) == b' '
    };
}

#[macro_export]
macro_rules! IS_SPACE {
    ($string:expr) => {
        $crate::IS_SPACE_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_TAB_AT {
    ($string:expr, $offset:expr) => {
        *$string.pointer.add($offset as usize) == b'\t'
    };
}

#[macro_export]
macro_rules! IS_TAB {
    ($string:expr) => {
        $crate::IS_TAB_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_BLANK_AT {
    ($string:expr, $offset:expr) => {
        $crate::IS_SPACE_AT!($string, $offset) || $crate::IS_TAB_AT!($string, $offset)
    };
}

#[macro_export]
macro_rules! IS_BLANK {
    ($string:expr) => {
        $crate::IS_BLANK_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_BREAK_AT {
    ($string:expr, $offset:expr) => {{
        let p = $string.pointer.add($offset as usize);
        *p == b'\r'
            || *p == b'\n'
            || (*p == 0xC2u8 && *p.add(1) == 0x85u8)
            || (*p == 0xE2u8 && *p.add(1) == 0x80u8 && *p.add(2) == 0xA8u8)
            || (*p == 0xE2u8 && *p.add(1) == 0x80u8 && *p.add(2) == 0xA9u8)
    }};
}

#[macro_export]
macro_rules! IS_BREAK {
    ($string:expr) => {
        $crate::IS_BREAK_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_CRLF_AT {
    ($string:expr, $offset:expr) => {
        *$string.pointer.add($offset as usize) == b'\r'
            && *$string.pointer.add($offset as usize + 1) == b'\n'
    };
}

#[macro_export]
macro_rules! IS_CRLF {
    ($string:expr) => {
        $crate::IS_CRLF_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_BREAKZ_AT {
    ($string:expr, $offset:expr) => {
        $crate::IS_BREAK_AT!($string, $offset) || $crate::IS_Z_AT!($string, $offset)
    };
}

#[macro_export]
macro_rules! IS_BREAKZ {
    ($string:expr) => {
        $crate::IS_BREAKZ_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_SPACEZ_AT {
    ($string:expr, $offset:expr) => {
        $crate::IS_SPACE_AT!($string, $offset) || $crate::IS_BREAKZ_AT!($string, $offset)
    };
}

#[macro_export]
macro_rules! IS_SPACEZ {
    ($string:expr) => {
        $crate::IS_SPACEZ_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! IS_BLANKZ_AT {
    ($string:expr, $offset:expr) => {
        $crate::IS_BLANK_AT!($string, $offset) || $crate::IS_BREAKZ_AT!($string, $offset)
    };
}

#[macro_export]
macro_rules! IS_BLANKZ {
    ($string:expr) => {
        $crate::IS_BLANKZ_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! WIDTH_AT {
    ($string:expr, $offset:expr) => {{
        let c = *$string.pointer.add($offset as usize);
        if (c & 0x80) == 0x00 {
            1usize
        } else if (c & 0xE0) == 0xC0 {
            2usize
        } else if (c & 0xF0) == 0xE0 {
            3usize
        } else if (c & 0xF8) == 0xF0 {
            4usize
        } else {
            0usize
        }
    }};
}

#[macro_export]
macro_rules! WIDTH {
    ($string:expr) => {
        $crate::WIDTH_AT!($string, 0)
    };
}

#[macro_export]
macro_rules! MOVE {
    ($string:expr) => {{
        let w = $crate::WIDTH!($string);
        $string.pointer = $string.pointer.add(w);
    }};
}

#[macro_export]
macro_rules! COPY {
    ($string_a:expr, $string_b:expr) => {{
        let c = *$string_b.pointer;
        if (c & 0x80) == 0x00 {
            *$string_a.pointer = *$string_b.pointer;
            $string_a.pointer = $string_a.pointer.add(1);
            $string_b.pointer = $string_b.pointer.add(1);
        } else if (c & 0xE0) == 0xC0 {
            *$string_a.pointer = *$string_b.pointer;
            $string_a.pointer = $string_a.pointer.add(1);
            $string_b.pointer = $string_b.pointer.add(1);
            *$string_a.pointer = *$string_b.pointer;
            $string_a.pointer = $string_a.pointer.add(1);
            $string_b.pointer = $string_b.pointer.add(1);
        } else if (c & 0xF0) == 0xE0 {
            *$string_a.pointer = *$string_b.pointer;
            $string_a.pointer = $string_a.pointer.add(1);
            $string_b.pointer = $string_b.pointer.add(1);
            *$string_a.pointer = *$string_b.pointer;
            $string_a.pointer = $string_a.pointer.add(1);
            $string_b.pointer = $string_b.pointer.add(1);
            *$string_a.pointer = *$string_b.pointer;
            $string_a.pointer = $string_a.pointer.add(1);
            $string_b.pointer = $string_b.pointer.add(1);
        } else if (c & 0xF8) == 0xF0 {
            *$string_a.pointer = *$string_b.pointer;
            $string_a.pointer = $string_a.pointer.add(1);
            $string_b.pointer = $string_b.pointer.add(1);
            *$string_a.pointer = *$string_b.pointer;
            $string_a.pointer = $string_a.pointer.add(1);
            $string_b.pointer = $string_b.pointer.add(1);
            *$string_a.pointer = *$string_b.pointer;
            $string_a.pointer = $string_a.pointer.add(1);
            $string_b.pointer = $string_b.pointer.add(1);
            *$string_a.pointer = *$string_b.pointer;
            $string_a.pointer = $string_a.pointer.add(1);
            $string_b.pointer = $string_b.pointer.add(1);
        }
    }};
}

// Stack macros

#[macro_export]
macro_rules! STACK_INIT {
    ($context:expr, $stack:expr, $T:ty) => {{
        let sz = $crate::INITIAL_STACK_SIZE * core::mem::size_of::<$T>();
        let ptr = $crate::api::yaml_malloc(sz) as *mut $T;
        if !ptr.is_null() {
            $stack.start = ptr;
            $stack.top = ptr;
            $stack.end = ptr.add($crate::INITIAL_STACK_SIZE);
            1i32
        } else {
            (*$context).error = $crate::YAML_MEMORY_ERROR;
            0i32
        }
    }};
}

#[macro_export]
macro_rules! STACK_DEL {
    ($context:expr, $stack:expr) => {{
        $crate::api::yaml_free($stack.start as *mut libc::c_void);
        $stack.start = core::ptr::null_mut();
        $stack.top = core::ptr::null_mut();
        $stack.end = core::ptr::null_mut();
    }};
}

#[macro_export]
macro_rules! STACK_EMPTY {
    ($context:expr, $stack:expr) => {
        $stack.start == $stack.top
    };
}

#[macro_export]
macro_rules! STACK_LIMIT {
    ($context:expr, $stack:expr, $size:expr) => {{
        if $stack.top.offset_from($stack.start) < $size as isize {
            1i32
        } else {
            (*$context).error = $crate::YAML_MEMORY_ERROR;
            0i32
        }
    }};
}

#[macro_export]
macro_rules! PUSH {
    ($context:expr, $stack:expr, $value:expr) => {{
        if $stack.top != $stack.end
            || $crate::api::yaml_stack_extend(
                &mut $stack.start as *mut _ as *mut *mut libc::c_void,
                &mut $stack.top as *mut _ as *mut *mut libc::c_void,
                &mut $stack.end as *mut _ as *mut *mut libc::c_void,
            ) != 0
        {
            *$stack.top = $value;
            $stack.top = $stack.top.add(1);
            1i32
        } else {
            (*$context).error = $crate::YAML_MEMORY_ERROR;
            0i32
        }
    }};
}

#[macro_export]
macro_rules! POP {
    ($context:expr, $stack:expr) => {{
        $stack.top = $stack.top.sub(1);
        *$stack.top
    }};
}

#[macro_export]
macro_rules! QUEUE_INIT {
    ($context:expr, $queue:expr, $size:expr, $T:ty) => {{
        let sz = $size * core::mem::size_of::<$T>();
        let ptr = $crate::api::yaml_malloc(sz) as *mut $T;
        if !ptr.is_null() {
            $queue.start = ptr;
            $queue.head = ptr;
            $queue.tail = ptr;
            $queue.end = ptr.add($size);
            1i32
        } else {
            (*$context).error = $crate::YAML_MEMORY_ERROR;
            0i32
        }
    }};
}

#[macro_export]
macro_rules! QUEUE_DEL {
    ($context:expr, $queue:expr) => {{
        $crate::api::yaml_free($queue.start as *mut libc::c_void);
        $queue.start = core::ptr::null_mut();
        $queue.head = core::ptr::null_mut();
        $queue.tail = core::ptr::null_mut();
        $queue.end = core::ptr::null_mut();
    }};
}

#[macro_export]
macro_rules! QUEUE_EMPTY {
    ($context:expr, $queue:expr) => {
        $queue.head == $queue.tail
    };
}

#[macro_export]
macro_rules! ENQUEUE {
    ($context:expr, $queue:expr, $value:expr) => {{
        if $queue.tail != $queue.end
            || $crate::api::yaml_queue_extend(
                &mut $queue.start as *mut _ as *mut *mut libc::c_void,
                &mut $queue.head as *mut _ as *mut *mut libc::c_void,
                &mut $queue.tail as *mut _ as *mut *mut libc::c_void,
                &mut $queue.end as *mut _ as *mut *mut libc::c_void,
            ) != 0
        {
            *$queue.tail = $value;
            $queue.tail = $queue.tail.add(1);
            1i32
        } else {
            (*$context).error = $crate::YAML_MEMORY_ERROR;
            0i32
        }
    }};
}

#[macro_export]
macro_rules! DEQUEUE {
    ($context:expr, $queue:expr) => {{
        let val = *$queue.head;
        $queue.head = $queue.head.add(1);
        val
    }};
}

#[macro_export]
macro_rules! QUEUE_INSERT {
    ($context:expr, $queue:expr, $index:expr, $value:expr) => {{
        if $queue.tail != $queue.end
            || $crate::api::yaml_queue_extend(
                &mut $queue.start as *mut _ as *mut *mut libc::c_void,
                &mut $queue.head as *mut _ as *mut *mut libc::c_void,
                &mut $queue.tail as *mut _ as *mut *mut libc::c_void,
                &mut $queue.end as *mut _ as *mut *mut libc::c_void,
            ) != 0
        {
            let idx = $index as usize;
            let count = $queue.tail.offset_from($queue.head.add(idx)) as usize;
            libc::memmove(
                $queue.head.add(idx + 1) as *mut libc::c_void,
                $queue.head.add(idx) as *const libc::c_void,
                count * core::mem::size_of_val(&*$queue.head),
            );
            *$queue.head.add(idx) = $value;
            $queue.tail = $queue.tail.add(1);
            1i32
        } else {
            (*$context).error = $crate::YAML_MEMORY_ERROR;
            0i32
        }
    }};
}

// Scanner macros

#[macro_export]
macro_rules! CACHE {
    ($parser:expr, $length:expr) => {{
        if (*$parser).unread >= $length as usize {
            1i32
        } else {
            $crate::reader::yaml_parser_update_buffer($parser, $length as usize)
        }
    }};
}

#[macro_export]
macro_rules! SKIP {
    ($parser:expr) => {{
        (*$parser).mark.index += 1;
        (*$parser).mark.column += 1;
        (*$parser).unread -= 1;
        let w = $crate::WIDTH!((*$parser).buffer);
        (*$parser).buffer.pointer = (*$parser).buffer.pointer.add(w);
    }};
}

#[macro_export]
macro_rules! SKIP_LINE {
    ($parser:expr) => {{
        if $crate::IS_CRLF!((*$parser).buffer) {
            (*$parser).mark.index += 2;
            (*$parser).mark.column = 0;
            (*$parser).mark.line += 1;
            (*$parser).unread -= 2;
            (*$parser).buffer.pointer = (*$parser).buffer.pointer.add(2);
        } else if $crate::IS_BREAK!((*$parser).buffer) {
            (*$parser).mark.index += 1;
            (*$parser).mark.column = 0;
            (*$parser).mark.line += 1;
            (*$parser).unread -= 1;
            let w = $crate::WIDTH!((*$parser).buffer);
            (*$parser).buffer.pointer = (*$parser).buffer.pointer.add(w);
        }
    }};
}

#[macro_export]
macro_rules! READ {
    ($parser:expr, $string:expr) => {{
        if STRING_EXTEND!($parser, $string) != 0 {
            COPY!($string, (*$parser).buffer);
            (*$parser).mark.index += 1;
            if $crate::IS_BREAK!((*$parser).buffer) {
                (*$parser).mark.column = 0;
                (*$parser).mark.line += 1;
            } else {
                (*$parser).mark.column += 1;
            }
            (*$parser).unread -= 1;
            1i32
        } else {
            0i32
        }
    }};
}

#[macro_export]
macro_rules! READ_LINE {
    ($parser:expr, $string:expr) => {{
        if STRING_EXTEND!($parser, $string) != 0 {
            if $crate::IS_CRLF!((*$parser).buffer) {
                *$string.pointer = b'\n';
                $string.pointer = $string.pointer.add(1);
                (*$parser).buffer.pointer = (*$parser).buffer.pointer.add(2);
                (*$parser).mark.index += 2;
                (*$parser).mark.column = 0;
                (*$parser).mark.line += 1;
                (*$parser).unread -= 2;
            } else if $crate::IS_BREAK!((*$parser).buffer) {
                *$string.pointer = b'\n';
                $string.pointer = $string.pointer.add(1);
                let w = $crate::WIDTH!((*$parser).buffer);
                (*$parser).buffer.pointer = (*$parser).buffer.pointer.add(w);
                (*$parser).mark.index += 1;
                (*$parser).mark.column = 0;
                (*$parser).mark.line += 1;
                (*$parser).unread -= 1;
            } else {
                *$string.pointer = *(*$parser).buffer.pointer;
                $string.pointer = $string.pointer.add(1);
                (*$parser).buffer.pointer = (*$parser).buffer.pointer.add(1);
                (*$parser).mark.index += 1;
                (*$parser).mark.column += 1;
                (*$parser).unread -= 1;
            }
            1i32
        } else {
            0i32
        }
    }};
}

// Emitter macros

#[macro_export]
macro_rules! FLUSH {
    ($emitter:expr) => {{
        if (*$emitter).buffer.pointer.add(5) < (*$emitter).buffer.end {
            1i32
        } else {
            $crate::writer::yaml_emitter_flush($emitter)
        }
    }};
}

#[macro_export]
macro_rules! PUT {
    ($emitter:expr, $value:expr) => {{
        if $crate::FLUSH!($emitter) != 0 {
            *(*$emitter).buffer.pointer = $value as u8;
            (*$emitter).buffer.pointer = (*$emitter).buffer.pointer.add(1);
            (*$emitter).column += 1;
            1i32
        } else {
            0i32
        }
    }};
}

#[macro_export]
macro_rules! PUT_BREAK {
    ($emitter:expr) => {{
        if $crate::FLUSH!($emitter) != 0 {
            if (*$emitter).line_break == $crate::YAML_CR_BREAK {
                *(*$emitter).buffer.pointer = b'\r';
                (*$emitter).buffer.pointer = (*$emitter).buffer.pointer.add(1);
                (*$emitter).column = 0;
                let old = (*$emitter).line;
                (*$emitter).line += 1;
                old
            } else if (*$emitter).line_break == $crate::YAML_LN_BREAK {
                *(*$emitter).buffer.pointer = b'\n';
                (*$emitter).buffer.pointer = (*$emitter).buffer.pointer.add(1);
                (*$emitter).column = 0;
                let old = (*$emitter).line;
                (*$emitter).line += 1;
                old
            } else if (*$emitter).line_break == $crate::YAML_CRLN_BREAK {
                *(*$emitter).buffer.pointer = b'\r';
                (*$emitter).buffer.pointer = (*$emitter).buffer.pointer.add(1);
                *(*$emitter).buffer.pointer = b'\n';
                (*$emitter).buffer.pointer = (*$emitter).buffer.pointer.add(1);
                (*$emitter).column = 0;
                let old = (*$emitter).line;
                (*$emitter).line += 1;
                old
            } else {
                0i32
            }
        } else {
            0i32
        }
    }};
}

#[macro_export]
macro_rules! WRITE {
    ($emitter:expr, $string:expr) => {{
        if $crate::FLUSH!($emitter) != 0 {
            $crate::COPY!((*$emitter).buffer, $string);
            (*$emitter).column += 1;
            1i32
        } else {
            0i32
        }
    }};
}

#[macro_export]
macro_rules! WRITE_BREAK {
    ($emitter:expr, $string:expr) => {{
        if $crate::FLUSH!($emitter) != 0 {
            if $crate::CHECK!($string, b'\n') {
                $crate::PUT_BREAK!($emitter)
            } else {
                $crate::COPY!((*$emitter).buffer, $string);
                (*$emitter).column = 0;
                let old = (*$emitter).line;
                (*$emitter).line += 1;
                old
            }
        } else {
            0i32
        }
    }};
}

// Token initializer macros

#[macro_export]
macro_rules! TOKEN_INIT {
    ($token:expr, $token_type:expr, $token_start_mark:expr, $token_end_mark:expr) => {{
        libc::memset(
            &mut $token as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<$crate::yaml_token_t>(),
        );
        $token.type_ = $token_type;
        $token.start_mark = $token_start_mark;
        $token.end_mark = $token_end_mark;
    }};
}

#[macro_export]
macro_rules! EVENT_INIT {
    ($event:expr, $event_type:expr, $event_start_mark:expr, $event_end_mark:expr) => {{
        libc::memset(
            &mut $event as *mut _ as *mut libc::c_void,
            0,
            core::mem::size_of::<$crate::yaml_event_t>(),
        );
        $event.type_ = $event_type;
        $event.start_mark = $event_start_mark;
        $event.end_mark = $event_end_mark;
    }};
}

// YAML_MALLOC helpers
#[inline(always)]
pub unsafe fn YAML_MALLOC_STATIC<T>() -> *mut T {
    crate::api::yaml_malloc(core::mem::size_of::<T>()) as *mut T
}

#[inline(always)]
pub unsafe fn YAML_MALLOC(size: usize) -> *mut yaml_char_t {
    crate::api::yaml_malloc(size) as *mut yaml_char_t
}
