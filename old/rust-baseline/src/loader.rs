use crate::*;
use libc::{c_char, c_int, size_t};

// Local node init macros (mirrors those in api.rs)
macro_rules! NODE_INIT {
    ($node:expr, $node_type:expr, $node_tag:expr, $start_mark:expr, $end_mark:expr) => {
        libc::memset(
            &mut $node as *mut _ as *mut libc::c_void,
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

// Document loading context
#[repr(C)]
pub struct loader_ctx {
    pub start: *mut c_int,
    pub end: *mut c_int,
    pub top: *mut c_int,
}

/*
 * Load the next document of the stream.
 */
#[no_mangle]
pub unsafe extern "C" fn yaml_parser_load(
    parser: *mut yaml_parser_t,
    document: *mut yaml_document_t,
) -> c_int {
    let mut event: yaml_event_t = core::mem::zeroed();

    assert!(!parser.is_null());
    assert!(!document.is_null());

    libc::memset(
        document as *mut libc::c_void,
        0,
        core::mem::size_of::<yaml_document_t>(),
    );
    if STACK_INIT!(parser, (*document).nodes, yaml_node_t) == 0 {
        // goto error
        yaml_parser_delete_aliases(parser);
        crate::api::yaml_document_delete(document);
        (*parser).document = core::ptr::null_mut();
        return 0;
    }

    if (*parser).stream_start_produced == 0 {
        if crate::parser::yaml_parser_parse(parser, &mut event) == 0 {
            // goto error
            yaml_parser_delete_aliases(parser);
            crate::api::yaml_document_delete(document);
            (*parser).document = core::ptr::null_mut();
            return 0;
        }
        assert!(event.type_ == YAML_STREAM_START_EVENT);
    }

    if (*parser).stream_end_produced != 0 {
        return 1;
    }

    if crate::parser::yaml_parser_parse(parser, &mut event) == 0 {
        // goto error
        yaml_parser_delete_aliases(parser);
        crate::api::yaml_document_delete(document);
        (*parser).document = core::ptr::null_mut();
        return 0;
    }
    if event.type_ == YAML_STREAM_END_EVENT {
        return 1;
    }

    if STACK_INIT!(parser, (*parser).aliases, yaml_alias_data_t) == 0 {
        // goto error
        yaml_parser_delete_aliases(parser);
        crate::api::yaml_document_delete(document);
        (*parser).document = core::ptr::null_mut();
        return 0;
    }

    (*parser).document = document;

    if yaml_parser_load_document(parser, &mut event) == 0 {
        // goto error
        yaml_parser_delete_aliases(parser);
        crate::api::yaml_document_delete(document);
        (*parser).document = core::ptr::null_mut();
        return 0;
    }

    yaml_parser_delete_aliases(parser);
    (*parser).document = core::ptr::null_mut();

    return 1;
}

/*
 * Set composer error.
 */
unsafe fn yaml_parser_set_composer_error(
    parser: *mut yaml_parser_t,
    problem: *const c_char,
    problem_mark: yaml_mark_t,
) -> c_int {
    (*parser).error = YAML_COMPOSER_ERROR;
    (*parser).problem = problem;
    (*parser).problem_mark = problem_mark;
    return 0;
}

/*
 * Set composer error with context.
 */
unsafe fn yaml_parser_set_composer_error_context(
    parser: *mut yaml_parser_t,
    context: *const c_char,
    context_mark: yaml_mark_t,
    problem: *const c_char,
    problem_mark: yaml_mark_t,
) -> c_int {
    (*parser).error = YAML_COMPOSER_ERROR;
    (*parser).context = context;
    (*parser).context_mark = context_mark;
    (*parser).problem = problem;
    (*parser).problem_mark = problem_mark;
    return 0;
}

/*
 * Delete the stack of aliases.
 */
unsafe fn yaml_parser_delete_aliases(parser: *mut yaml_parser_t) {
    while !STACK_EMPTY!(parser, (*parser).aliases) {
        crate::api::yaml_free(POP!(parser, (*parser).aliases).anchor as *mut libc::c_void);
    }
    STACK_DEL!(parser, (*parser).aliases);
}

/*
 * Compose a document object.
 */
unsafe fn yaml_parser_load_document(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
) -> c_int {
    let mut ctx = loader_ctx {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        top: core::ptr::null_mut(),
    };

    assert!((*event).type_ == YAML_DOCUMENT_START_EVENT);

    (*(*parser).document).version_directive =
        (*event).data.document_start.version_directive;
    (*(*parser).document).tag_directives.start =
        (*event).data.document_start.tag_directives.start;
    (*(*parser).document).tag_directives.end =
        (*event).data.document_start.tag_directives.end;
    (*(*parser).document).start_implicit =
        (*event).data.document_start.implicit;
    (*(*parser).document).start_mark = (*event).start_mark;

    if STACK_INIT!(parser, ctx, c_int) == 0 {
        return 0;
    }
    if yaml_parser_load_nodes(parser, &mut ctx) == 0 {
        STACK_DEL!(parser, ctx);
        return 0;
    }
    STACK_DEL!(parser, ctx);

    return 1;
}

/*
 * Compose a node tree.
 */
unsafe fn yaml_parser_load_nodes(
    parser: *mut yaml_parser_t,
    ctx: *mut loader_ctx,
) -> c_int {
    let mut event: yaml_event_t = core::mem::zeroed();

    loop {
        if crate::parser::yaml_parser_parse(parser, &mut event) == 0 {
            return 0;
        }

        match event.type_ {
            YAML_ALIAS_EVENT => {
                if yaml_parser_load_alias(parser, &mut event, ctx) == 0 {
                    return 0;
                }
            }
            YAML_SCALAR_EVENT => {
                if yaml_parser_load_scalar(parser, &mut event, ctx) == 0 {
                    return 0;
                }
            }
            YAML_SEQUENCE_START_EVENT => {
                if yaml_parser_load_sequence(parser, &mut event, ctx) == 0 {
                    return 0;
                }
            }
            YAML_SEQUENCE_END_EVENT => {
                if yaml_parser_load_sequence_end(parser, &mut event, ctx) == 0 {
                    return 0;
                }
            }
            YAML_MAPPING_START_EVENT => {
                if yaml_parser_load_mapping(parser, &mut event, ctx) == 0 {
                    return 0;
                }
            }
            YAML_MAPPING_END_EVENT => {
                if yaml_parser_load_mapping_end(parser, &mut event, ctx) == 0 {
                    return 0;
                }
            }
            YAML_DOCUMENT_END_EVENT => {
                // break out below
            }
            _ => {
                assert!(false); /* Could not happen. */
                return 0;
            }
        }

        if event.type_ == YAML_DOCUMENT_END_EVENT {
            break;
        }
    }

    (*(*parser).document).end_implicit = event.data.document_end.implicit;
    (*(*parser).document).end_mark = event.end_mark;

    return 1;
}

/*
 * Add an anchor.
 */
unsafe fn yaml_parser_register_anchor(
    parser: *mut yaml_parser_t,
    index: c_int,
    anchor: *mut yaml_char_t,
) -> c_int {
    let mut data: yaml_alias_data_t = core::mem::zeroed();

    if anchor.is_null() {
        return 1;
    }

    data.anchor = anchor;
    data.index = index;
    data.mark = (*(*parser).document).nodes.start.offset((index - 1) as isize).read().start_mark;

    let mut alias_data = (*parser).aliases.start;
    while alias_data != (*parser).aliases.top {
        if libc::strcmp((*alias_data).anchor as *const c_char, anchor as *const c_char) == 0 {
            crate::api::yaml_free(anchor as *mut libc::c_void);
            return yaml_parser_set_composer_error_context(
                parser,
                b"found duplicate anchor; first occurrence\0".as_ptr() as *const c_char,
                (*alias_data).mark,
                b"second occurrence\0".as_ptr() as *const c_char,
                data.mark,
            );
        }
        alias_data = alias_data.add(1);
    }

    if PUSH!(parser, (*parser).aliases, data) == 0 {
        crate::api::yaml_free(anchor as *mut libc::c_void);
        return 0;
    }

    return 1;
}

/*
 * Compose node into its parent in the tree.
 */
unsafe fn yaml_parser_load_node_add(
    parser: *mut yaml_parser_t,
    ctx: *mut loader_ctx,
    index: c_int,
) -> c_int {
    if STACK_EMPTY!(parser, *ctx) {
        return 1;
    }

    let parent_index = *((*ctx).top.sub(1));
    let parent = &mut *(*(*parser).document).nodes.start.offset((parent_index - 1) as isize);

    match parent.type_ {
        YAML_SEQUENCE_NODE => {
            if STACK_LIMIT!(parser, parent.data.sequence.items, libc::INT_MAX as isize - 1) == 0 {
                return 0;
            }
            if PUSH!(parser, parent.data.sequence.items, index) == 0 {
                return 0;
            }
        }
        YAML_MAPPING_NODE => {
            let mut pair: yaml_node_pair_t = core::mem::zeroed();
            if !STACK_EMPTY!(parser, parent.data.mapping.pairs) {
                let p = parent.data.mapping.pairs.top.sub(1);
                if (*p).key != 0 && (*p).value == 0 {
                    (*p).value = index;
                    return 1;
                }
            }

            pair.key = index;
            pair.value = 0;
            if STACK_LIMIT!(parser, parent.data.mapping.pairs, libc::INT_MAX as isize - 1) == 0 {
                return 0;
            }
            if PUSH!(parser, parent.data.mapping.pairs, pair) == 0 {
                return 0;
            }
        }
        _ => {
            assert!(false); /* Could not happen. */
            return 0;
        }
    }
    return 1;
}

/*
 * Compose a node corresponding to an alias.
 */
unsafe fn yaml_parser_load_alias(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    ctx: *mut loader_ctx,
) -> c_int {
    let anchor = (*event).data.alias.anchor;
    let mut alias_data = (*parser).aliases.start;

    while alias_data != (*parser).aliases.top {
        if libc::strcmp((*alias_data).anchor as *const c_char, anchor as *const c_char) == 0 {
            crate::api::yaml_free(anchor as *mut libc::c_void);
            return yaml_parser_load_node_add(parser, ctx, (*alias_data).index);
        }
        alias_data = alias_data.add(1);
    }

    crate::api::yaml_free(anchor as *mut libc::c_void);
    return yaml_parser_set_composer_error(
        parser,
        b"found undefined alias\0".as_ptr() as *const c_char,
        (*event).start_mark,
    );
}

/*
 * Compose a scalar node.
 */
unsafe fn yaml_parser_load_scalar(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    ctx: *mut loader_ctx,
) -> c_int {
    let mut node: yaml_node_t = core::mem::zeroed();
    let index: c_int;
    let mut tag = (*event).data.scalar.tag;

    'error: {
        if STACK_LIMIT!(parser, (*(*parser).document).nodes, libc::INT_MAX as isize - 1) == 0 {
            break 'error;
        }

        if tag.is_null()
            || libc::strcmp(tag as *const c_char, b"!\0".as_ptr() as *const c_char) == 0
        {
            crate::api::yaml_free(tag as *mut libc::c_void);
            tag = crate::api::yaml_strdup(
                crate::api::YAML_DEFAULT_SCALAR_TAG.as_ptr() as *mut yaml_char_t,
            );
            if tag.is_null() {
                break 'error;
            }
        }

        SCALAR_NODE_INIT!(
            node,
            tag,
            (*event).data.scalar.value,
            (*event).data.scalar.length,
            (*event).data.scalar.style,
            (*event).start_mark,
            (*event).end_mark
        );

        if PUSH!(parser, (*(*parser).document).nodes, node) == 0 {
            break 'error;
        }

        index = (*(*parser).document).nodes.top
            .offset_from((*(*parser).document).nodes.start) as c_int;

        if yaml_parser_register_anchor(parser, index, (*event).data.scalar.anchor) == 0 {
            return 0;
        }

        return yaml_parser_load_node_add(parser, ctx, index);
    }

    // error:
    crate::api::yaml_free(tag as *mut libc::c_void);
    crate::api::yaml_free((*event).data.scalar.anchor as *mut libc::c_void);
    crate::api::yaml_free((*event).data.scalar.value as *mut libc::c_void);
    return 0;
}

/*
 * Compose a sequence node.
 */
unsafe fn yaml_parser_load_sequence(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    ctx: *mut loader_ctx,
) -> c_int {
    let mut node: yaml_node_t = core::mem::zeroed();
    let mut items = crate::yaml_node_item_stack_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        top: core::ptr::null_mut(),
    };
    let index: c_int;
    let mut tag = (*event).data.sequence_start.tag;

    'error: {
        if STACK_LIMIT!(parser, (*(*parser).document).nodes, libc::INT_MAX as isize - 1) == 0 {
            break 'error;
        }

        if tag.is_null()
            || libc::strcmp(tag as *const c_char, b"!\0".as_ptr() as *const c_char) == 0
        {
            crate::api::yaml_free(tag as *mut libc::c_void);
            tag = crate::api::yaml_strdup(
                crate::api::YAML_DEFAULT_SEQUENCE_TAG.as_ptr() as *mut yaml_char_t,
            );
            if tag.is_null() {
                break 'error;
            }
        }

        if STACK_INIT!(parser, items, yaml_node_item_t) == 0 {
            break 'error;
        }

        SEQUENCE_NODE_INIT!(
            node,
            tag,
            items.start,
            items.end,
            (*event).data.sequence_start.style,
            (*event).start_mark,
            (*event).end_mark
        );

        if PUSH!(parser, (*(*parser).document).nodes, node) == 0 {
            break 'error;
        }

        index = (*(*parser).document).nodes.top
            .offset_from((*(*parser).document).nodes.start) as c_int;

        if yaml_parser_register_anchor(
            parser,
            index,
            (*event).data.sequence_start.anchor,
        ) == 0
        {
            return 0;
        }

        if yaml_parser_load_node_add(parser, ctx, index) == 0 {
            return 0;
        }

        if STACK_LIMIT!(parser, *ctx, libc::INT_MAX as isize - 1) == 0 {
            return 0;
        }
        if PUSH!(parser, *ctx, index) == 0 {
            return 0;
        }

        return 1;
    }

    // error:
    crate::api::yaml_free(tag as *mut libc::c_void);
    crate::api::yaml_free((*event).data.sequence_start.anchor as *mut libc::c_void);
    return 0;
}

unsafe fn yaml_parser_load_sequence_end(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    ctx: *mut loader_ctx,
) -> c_int {
    let index: c_int;

    assert!(((*ctx).top.offset_from((*ctx).start)) > 0);

    index = *((*ctx).top.sub(1));
    assert!(
        (*(*(*parser).document).nodes.start.offset((index - 1) as isize)).type_
            == YAML_SEQUENCE_NODE
    );
    (*(*(*parser).document).nodes.start.offset((index - 1) as isize)).end_mark =
        (*event).end_mark;

    let _ = POP!(parser, *ctx);

    return 1;
}

/*
 * Compose a mapping node.
 */
unsafe fn yaml_parser_load_mapping(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    ctx: *mut loader_ctx,
) -> c_int {
    let mut node: yaml_node_t = core::mem::zeroed();
    let mut pairs = crate::yaml_node_pair_stack_t {
        start: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        top: core::ptr::null_mut(),
    };
    let index: c_int;
    let mut tag = (*event).data.mapping_start.tag;

    'error: {
        if STACK_LIMIT!(parser, (*(*parser).document).nodes, libc::INT_MAX as isize - 1) == 0 {
            break 'error;
        }

        if tag.is_null()
            || libc::strcmp(tag as *const c_char, b"!\0".as_ptr() as *const c_char) == 0
        {
            crate::api::yaml_free(tag as *mut libc::c_void);
            tag = crate::api::yaml_strdup(
                crate::api::YAML_DEFAULT_MAPPING_TAG.as_ptr() as *mut yaml_char_t,
            );
            if tag.is_null() {
                break 'error;
            }
        }

        if STACK_INIT!(parser, pairs, yaml_node_pair_t) == 0 {
            break 'error;
        }

        MAPPING_NODE_INIT!(
            node,
            tag,
            pairs.start,
            pairs.end,
            (*event).data.mapping_start.style,
            (*event).start_mark,
            (*event).end_mark
        );

        if PUSH!(parser, (*(*parser).document).nodes, node) == 0 {
            break 'error;
        }

        index = (*(*parser).document).nodes.top
            .offset_from((*(*parser).document).nodes.start) as c_int;

        if yaml_parser_register_anchor(
            parser,
            index,
            (*event).data.mapping_start.anchor,
        ) == 0
        {
            return 0;
        }

        if yaml_parser_load_node_add(parser, ctx, index) == 0 {
            return 0;
        }

        if STACK_LIMIT!(parser, *ctx, libc::INT_MAX as isize - 1) == 0 {
            return 0;
        }
        if PUSH!(parser, *ctx, index) == 0 {
            return 0;
        }

        return 1;
    }

    // error:
    crate::api::yaml_free(tag as *mut libc::c_void);
    crate::api::yaml_free((*event).data.mapping_start.anchor as *mut libc::c_void);
    return 0;
}

unsafe fn yaml_parser_load_mapping_end(
    parser: *mut yaml_parser_t,
    event: *mut yaml_event_t,
    ctx: *mut loader_ctx,
) -> c_int {
    let index: c_int;

    assert!(((*ctx).top.offset_from((*ctx).start)) > 0);

    index = *((*ctx).top.sub(1));
    assert!(
        (*(*(*parser).document).nodes.start.offset((index - 1) as isize)).type_
            == YAML_MAPPING_NODE
    );
    (*(*(*parser).document).nodes.start.offset((index - 1) as isize)).end_mark =
        (*event).end_mark;

    let _ = POP!(parser, *ctx);

    return 1;
}
