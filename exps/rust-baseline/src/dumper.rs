use crate::*;
use libc::{c_char, c_int, size_t};

const ANCHOR_TEMPLATE: &[u8] = b"id%03d\0";
const ANCHOR_TEMPLATE_LENGTH: usize = 16;

// Local event init macros
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

/*
 * Issue a STREAM-START event.
 */
#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_open(emitter: *mut yaml_emitter_t) -> c_int {
    let mut event: yaml_event_t = core::mem::zeroed();
    let mark = yaml_mark_t {
        index: 0,
        line: 0,
        column: 0,
    };

    assert!(!emitter.is_null());
    assert!((*emitter).opened == 0);

    STREAM_START_EVENT_INIT!(event, YAML_ANY_ENCODING, mark, mark);

    if crate::emitter::yaml_emitter_emit(emitter, &mut event) == 0 {
        return 0;
    }

    (*emitter).opened = 1;

    return 1;
}

/*
 * Issue a STREAM-END event.
 */
#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_close(emitter: *mut yaml_emitter_t) -> c_int {
    let mut event: yaml_event_t = core::mem::zeroed();
    let mark = yaml_mark_t {
        index: 0,
        line: 0,
        column: 0,
    };

    assert!(!emitter.is_null());
    assert!((*emitter).opened != 0);

    if (*emitter).closed != 0 {
        return 1;
    }

    STREAM_END_EVENT_INIT!(event, mark, mark);

    if crate::emitter::yaml_emitter_emit(emitter, &mut event) == 0 {
        return 0;
    }

    (*emitter).closed = 1;

    return 1;
}

/*
 * Dump a YAML document.
 */
#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_dump(
    emitter: *mut yaml_emitter_t,
    document: *mut yaml_document_t,
) -> c_int {
    let mut event: yaml_event_t = core::mem::zeroed();
    let mark = yaml_mark_t {
        index: 0,
        line: 0,
        column: 0,
    };

    assert!(!emitter.is_null());
    assert!(!document.is_null());

    (*emitter).document = document;

    if (*emitter).opened == 0 {
        if yaml_emitter_open(emitter) == 0 {
            // goto error
            yaml_emitter_delete_document_and_anchors(emitter);
            return 0;
        }
    }

    if STACK_EMPTY!(emitter, (*document).nodes) {
        if yaml_emitter_close(emitter) == 0 {
            // goto error
            yaml_emitter_delete_document_and_anchors(emitter);
            return 0;
        }
        yaml_emitter_delete_document_and_anchors(emitter);
        return 1;
    }

    assert!((*emitter).opened != 0);

    (*emitter).anchors = crate::api::yaml_malloc(
        core::mem::size_of::<yaml_anchors_t>()
            * ((*document).nodes.top.offset_from((*document).nodes.start) as usize),
    ) as *mut yaml_anchors_t;
    if (*emitter).anchors.is_null() {
        // goto error
        yaml_emitter_delete_document_and_anchors(emitter);
        return 0;
    }
    libc::memset(
        (*emitter).anchors as *mut libc::c_void,
        0,
        core::mem::size_of::<yaml_anchors_t>()
            * ((*document).nodes.top.offset_from((*document).nodes.start) as usize),
    );

    DOCUMENT_START_EVENT_INIT!(
        event,
        (*document).version_directive,
        (*document).tag_directives.start,
        (*document).tag_directives.end,
        (*document).start_implicit,
        mark,
        mark
    );
    if crate::emitter::yaml_emitter_emit(emitter, &mut event) == 0 {
        // goto error
        yaml_emitter_delete_document_and_anchors(emitter);
        return 0;
    }

    yaml_emitter_anchor_node(emitter, 1);
    if yaml_emitter_dump_node(emitter, 1) == 0 {
        // goto error
        yaml_emitter_delete_document_and_anchors(emitter);
        return 0;
    }

    DOCUMENT_END_EVENT_INIT!(event, (*document).end_implicit, mark, mark);
    if crate::emitter::yaml_emitter_emit(emitter, &mut event) == 0 {
        // goto error
        yaml_emitter_delete_document_and_anchors(emitter);
        return 0;
    }

    yaml_emitter_delete_document_and_anchors(emitter);

    return 1;
}

/*
 * Clean up the emitter object after a document is dumped.
 */
unsafe fn yaml_emitter_delete_document_and_anchors(emitter: *mut yaml_emitter_t) {
    let mut index: c_int;

    if (*emitter).anchors.is_null() {
        crate::api::yaml_document_delete((*emitter).document);
        (*emitter).document = core::ptr::null_mut();
        return;
    }

    index = 0;
    while (*(*emitter).document).nodes.start.offset(index as isize)
        < (*(*emitter).document).nodes.top
    {
        let node: yaml_node_t =
            *(*(*emitter).document).nodes.start.offset(index as isize);
        if (*(*emitter).anchors.offset(index as isize)).serialized == 0 {
            crate::api::yaml_free(node.tag as *mut libc::c_void);
            if node.type_ == YAML_SCALAR_NODE {
                crate::api::yaml_free(node.data.scalar.value as *mut libc::c_void);
            }
        }
        if node.type_ == YAML_SEQUENCE_NODE {
            STACK_DEL!(emitter, (*(*(*emitter).document).nodes.start.offset(index as isize)).data.sequence.items);
        }
        if node.type_ == YAML_MAPPING_NODE {
            STACK_DEL!(emitter, (*(*(*emitter).document).nodes.start.offset(index as isize)).data.mapping.pairs);
        }
        index += 1;
    }

    STACK_DEL!(emitter, (*(*emitter).document).nodes);
    crate::api::yaml_free((*emitter).anchors as *mut libc::c_void);

    (*emitter).anchors = core::ptr::null_mut();
    (*emitter).last_anchor_id = 0;
    (*emitter).document = core::ptr::null_mut();
}

/*
 * Check the references of a node and assign the anchor id if needed.
 */
unsafe fn yaml_emitter_anchor_node(emitter: *mut yaml_emitter_t, index: c_int) {
    let node = (*(*emitter).document).nodes.start.offset((index - 1) as isize);

    (*(*emitter).anchors.offset((index - 1) as isize)).references += 1;

    if (*(*emitter).anchors.offset((index - 1) as isize)).references == 1 {
        match (*node).type_ {
            YAML_SEQUENCE_NODE => {
                let mut item = (*node).data.sequence.items.start;
                while item < (*node).data.sequence.items.top {
                    yaml_emitter_anchor_node(emitter, *item);
                    item = item.add(1);
                }
            }
            YAML_MAPPING_NODE => {
                let mut pair = (*node).data.mapping.pairs.start;
                while pair < (*node).data.mapping.pairs.top {
                    yaml_emitter_anchor_node(emitter, (*pair).key);
                    yaml_emitter_anchor_node(emitter, (*pair).value);
                    pair = pair.add(1);
                }
            }
            _ => {}
        }
    } else if (*(*emitter).anchors.offset((index - 1) as isize)).references == 2 {
        (*emitter).last_anchor_id += 1;
        (*(*emitter).anchors.offset((index - 1) as isize)).anchor = (*emitter).last_anchor_id;
    }
}

/*
 * Generate a textual representation for an anchor.
 */
unsafe fn yaml_emitter_generate_anchor(
    _emitter: *mut yaml_emitter_t,
    anchor_id: c_int,
) -> *mut yaml_char_t {
    let anchor = YAML_MALLOC(ANCHOR_TEMPLATE_LENGTH);

    if anchor.is_null() {
        return core::ptr::null_mut();
    }

    libc::sprintf(
        anchor as *mut c_char,
        ANCHOR_TEMPLATE.as_ptr() as *const c_char,
        anchor_id,
    );

    return anchor;
}

/*
 * Serialize a node.
 */
unsafe fn yaml_emitter_dump_node(emitter: *mut yaml_emitter_t, index: c_int) -> c_int {
    let node = (*(*emitter).document).nodes.start.offset((index - 1) as isize);
    let anchor_id = (*(*emitter).anchors.offset((index - 1) as isize)).anchor;
    let mut anchor: *mut yaml_char_t = core::ptr::null_mut();

    if anchor_id != 0 {
        anchor = yaml_emitter_generate_anchor(emitter, anchor_id);
        if anchor.is_null() {
            return 0;
        }
    }

    if (*(*emitter).anchors.offset((index - 1) as isize)).serialized != 0 {
        return yaml_emitter_dump_alias(emitter, anchor);
    }

    (*(*emitter).anchors.offset((index - 1) as isize)).serialized = 1;

    match (*node).type_ {
        YAML_SCALAR_NODE => {
            return yaml_emitter_dump_scalar(emitter, node, anchor);
        }
        YAML_SEQUENCE_NODE => {
            return yaml_emitter_dump_sequence(emitter, node, anchor);
        }
        YAML_MAPPING_NODE => {
            return yaml_emitter_dump_mapping(emitter, node, anchor);
        }
        _ => {
            assert!(false); /* Could not happen. */
        }
    }

    return 0; /* Could not happen. */
}

/*
 * Serialize an alias.
 */
unsafe fn yaml_emitter_dump_alias(
    emitter: *mut yaml_emitter_t,
    anchor: *mut yaml_char_t,
) -> c_int {
    let mut event: yaml_event_t = core::mem::zeroed();
    let mark = yaml_mark_t {
        index: 0,
        line: 0,
        column: 0,
    };

    ALIAS_EVENT_INIT!(event, anchor, mark, mark);

    return crate::emitter::yaml_emitter_emit(emitter, &mut event);
}

/*
 * Serialize a scalar.
 */
unsafe fn yaml_emitter_dump_scalar(
    emitter: *mut yaml_emitter_t,
    node: *mut yaml_node_t,
    anchor: *mut yaml_char_t,
) -> c_int {
    let mut event: yaml_event_t = core::mem::zeroed();
    let mark = yaml_mark_t {
        index: 0,
        line: 0,
        column: 0,
    };

    let plain_implicit = (libc::strcmp(
        (*node).tag as *const c_char,
        crate::api::YAML_DEFAULT_SCALAR_TAG.as_ptr() as *const c_char,
    ) == 0) as c_int;
    let quoted_implicit = (libc::strcmp(
        (*node).tag as *const c_char,
        crate::api::YAML_DEFAULT_SCALAR_TAG.as_ptr() as *const c_char,
    ) == 0) as c_int;

    SCALAR_EVENT_INIT!(
        event,
        anchor,
        (*node).tag,
        (*node).data.scalar.value,
        (*node).data.scalar.length,
        plain_implicit,
        quoted_implicit,
        (*node).data.scalar.style,
        mark,
        mark
    );

    return crate::emitter::yaml_emitter_emit(emitter, &mut event);
}

/*
 * Serialize a sequence.
 */
unsafe fn yaml_emitter_dump_sequence(
    emitter: *mut yaml_emitter_t,
    node: *mut yaml_node_t,
    anchor: *mut yaml_char_t,
) -> c_int {
    let mut event: yaml_event_t = core::mem::zeroed();
    let mark = yaml_mark_t {
        index: 0,
        line: 0,
        column: 0,
    };

    let implicit = (libc::strcmp(
        (*node).tag as *const c_char,
        crate::api::YAML_DEFAULT_SEQUENCE_TAG.as_ptr() as *const c_char,
    ) == 0) as c_int;

    SEQUENCE_START_EVENT_INIT!(
        event,
        anchor,
        (*node).tag,
        implicit,
        (*node).data.sequence.style,
        mark,
        mark
    );
    if crate::emitter::yaml_emitter_emit(emitter, &mut event) == 0 {
        return 0;
    }

    let mut item = (*node).data.sequence.items.start;
    while item < (*node).data.sequence.items.top {
        if yaml_emitter_dump_node(emitter, *item) == 0 {
            return 0;
        }
        item = item.add(1);
    }

    SEQUENCE_END_EVENT_INIT!(event, mark, mark);
    if crate::emitter::yaml_emitter_emit(emitter, &mut event) == 0 {
        return 0;
    }

    return 1;
}

/*
 * Serialize a mapping.
 */
unsafe fn yaml_emitter_dump_mapping(
    emitter: *mut yaml_emitter_t,
    node: *mut yaml_node_t,
    anchor: *mut yaml_char_t,
) -> c_int {
    let mut event: yaml_event_t = core::mem::zeroed();
    let mark = yaml_mark_t {
        index: 0,
        line: 0,
        column: 0,
    };

    let implicit = (libc::strcmp(
        (*node).tag as *const c_char,
        crate::api::YAML_DEFAULT_MAPPING_TAG.as_ptr() as *const c_char,
    ) == 0) as c_int;

    MAPPING_START_EVENT_INIT!(
        event,
        anchor,
        (*node).tag,
        implicit,
        (*node).data.mapping.style,
        mark,
        mark
    );
    if crate::emitter::yaml_emitter_emit(emitter, &mut event) == 0 {
        return 0;
    }

    let mut pair = (*node).data.mapping.pairs.start;
    while pair < (*node).data.mapping.pairs.top {
        if yaml_emitter_dump_node(emitter, (*pair).key) == 0 {
            return 0;
        }
        if yaml_emitter_dump_node(emitter, (*pair).value) == 0 {
            return 0;
        }
        pair = pair.add(1);
    }

    MAPPING_END_EVENT_INIT!(event, mark, mark);
    if crate::emitter::yaml_emitter_emit(emitter, &mut event) == 0 {
        return 0;
    }

    return 1;
}
