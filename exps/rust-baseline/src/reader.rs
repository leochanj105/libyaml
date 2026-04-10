use crate::*;
use libc::{c_char, c_int, size_t};

const BOM_UTF8: [u8; 3] = [0xEF, 0xBB, 0xBF];
const BOM_UTF16LE: [u8; 2] = [0xFF, 0xFE];
const BOM_UTF16BE: [u8; 2] = [0xFE, 0xFF];

unsafe fn yaml_parser_set_reader_error(
    parser: *mut yaml_parser_t,
    problem: *const c_char,
    offset: size_t,
    value: c_int,
) -> c_int {
    (*parser).error = YAML_READER_ERROR;
    (*parser).problem = problem;
    (*parser).problem_offset = offset;
    (*parser).problem_value = value;
    0
}

unsafe fn yaml_parser_determine_encoding(parser: *mut yaml_parser_t) -> c_int {
    while (*parser).eof == 0
        && (*parser).raw_buffer.last.offset_from((*parser).raw_buffer.pointer) < 3
    {
        if yaml_parser_update_raw_buffer(parser) == 0 {
            return 0;
        }
    }

    let raw_avail =
        (*parser).raw_buffer.last.offset_from((*parser).raw_buffer.pointer) as usize;

    if raw_avail >= 2
        && libc::memcmp(
            (*parser).raw_buffer.pointer as *const libc::c_void,
            BOM_UTF16LE.as_ptr() as *const libc::c_void,
            2,
        ) == 0
    {
        (*parser).encoding = YAML_UTF16LE_ENCODING;
        (*parser).raw_buffer.pointer = (*parser).raw_buffer.pointer.add(2);
        (*parser).offset += 2;
    } else if raw_avail >= 2
        && libc::memcmp(
            (*parser).raw_buffer.pointer as *const libc::c_void,
            BOM_UTF16BE.as_ptr() as *const libc::c_void,
            2,
        ) == 0
    {
        (*parser).encoding = YAML_UTF16BE_ENCODING;
        (*parser).raw_buffer.pointer = (*parser).raw_buffer.pointer.add(2);
        (*parser).offset += 2;
    } else if raw_avail >= 3
        && libc::memcmp(
            (*parser).raw_buffer.pointer as *const libc::c_void,
            BOM_UTF8.as_ptr() as *const libc::c_void,
            3,
        ) == 0
    {
        (*parser).encoding = YAML_UTF8_ENCODING;
        (*parser).raw_buffer.pointer = (*parser).raw_buffer.pointer.add(3);
        (*parser).offset += 3;
    } else {
        (*parser).encoding = YAML_UTF8_ENCODING;
    }

    1
}

unsafe fn yaml_parser_update_raw_buffer(parser: *mut yaml_parser_t) -> c_int {
    let mut size_read: size_t = 0;

    // Return if the raw buffer is full
    if (*parser).raw_buffer.start == (*parser).raw_buffer.pointer
        && (*parser).raw_buffer.last == (*parser).raw_buffer.end
    {
        return 1;
    }

    // Return on EOF
    if (*parser).eof != 0 {
        return 1;
    }

    // Move remaining bytes to beginning
    if (*parser).raw_buffer.start < (*parser).raw_buffer.pointer
        && (*parser).raw_buffer.pointer < (*parser).raw_buffer.last
    {
        let remaining = (*parser)
            .raw_buffer
            .last
            .offset_from((*parser).raw_buffer.pointer) as usize;
        libc::memmove(
            (*parser).raw_buffer.start as *mut libc::c_void,
            (*parser).raw_buffer.pointer as *const libc::c_void,
            remaining,
        );
    }
    (*parser).raw_buffer.last = (*parser).raw_buffer.last.sub(
        (*parser)
            .raw_buffer
            .pointer
            .offset_from((*parser).raw_buffer.start) as usize,
    );
    (*parser).raw_buffer.pointer = (*parser).raw_buffer.start;

    // Call the read handler
    let avail = (*parser)
        .raw_buffer
        .end
        .offset_from((*parser).raw_buffer.last) as usize;
    if ((*parser).read_handler.unwrap())(
        (*parser).read_handler_data,
        (*parser).raw_buffer.last,
        avail,
        &mut size_read,
    ) == 0
    {
        return yaml_parser_set_reader_error(
            parser,
            b"input error\0".as_ptr() as *const c_char,
            (*parser).offset,
            -1,
        );
    }
    (*parser).raw_buffer.last = (*parser).raw_buffer.last.add(size_read);
    if size_read == 0 {
        (*parser).eof = 1;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn yaml_parser_update_buffer(
    parser: *mut yaml_parser_t,
    length: size_t,
) -> c_int {
    let mut first: c_int = 1;

    debug_assert!((*parser).read_handler.is_some());

    // If EOF and raw buffer empty, do nothing
    if (*parser).eof != 0 && (*parser).raw_buffer.pointer == (*parser).raw_buffer.last {
        return 1;
    }

    // Return if buffer already has enough characters
    if (*parser).unread >= length {
        return 1;
    }

    // Determine encoding if not set
    if (*parser).encoding == YAML_ANY_ENCODING {
        if yaml_parser_determine_encoding(parser) == 0 {
            return 0;
        }
    }

    // Move unread characters to beginning of buffer
    if (*parser).buffer.start < (*parser).buffer.pointer
        && (*parser).buffer.pointer < (*parser).buffer.last
    {
        let size = (*parser)
            .buffer
            .last
            .offset_from((*parser).buffer.pointer) as usize;
        libc::memmove(
            (*parser).buffer.start as *mut libc::c_void,
            (*parser).buffer.pointer as *const libc::c_void,
            size,
        );
        (*parser).buffer.pointer = (*parser).buffer.start;
        (*parser).buffer.last = (*parser).buffer.start.add(size);
    } else if (*parser).buffer.pointer == (*parser).buffer.last {
        (*parser).buffer.pointer = (*parser).buffer.start;
        (*parser).buffer.last = (*parser).buffer.start;
    }

    // Fill buffer until it has enough characters
    while (*parser).unread < length {
        // Fill raw buffer if necessary
        if first == 0 || (*parser).raw_buffer.pointer == (*parser).raw_buffer.last {
            if yaml_parser_update_raw_buffer(parser) == 0 {
                return 0;
            }
        }
        first = 0;

        // Decode the raw buffer
        while (*parser).raw_buffer.pointer != (*parser).raw_buffer.last {
            let mut value: u32 = 0;
            let mut value2: u32 = 0;
            let mut incomplete: c_int = 0;
            let mut width: usize = 0;
            let raw_unread = (*parser)
                .raw_buffer
                .last
                .offset_from((*parser).raw_buffer.pointer) as usize;

            match (*parser).encoding {
                YAML_UTF8_ENCODING => {
                    let octet = *(*parser).raw_buffer.pointer;

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
                        return yaml_parser_set_reader_error(
                            parser,
                            b"invalid leading UTF-8 octet\0".as_ptr() as *const c_char,
                            (*parser).offset,
                            octet as c_int,
                        );
                    }

                    if width > raw_unread {
                        if (*parser).eof != 0 {
                            return yaml_parser_set_reader_error(
                                parser,
                                b"incomplete UTF-8 octet sequence\0".as_ptr() as *const c_char,
                                (*parser).offset,
                                -1,
                            );
                        }
                        incomplete = 1;
                        break;
                    }

                    value = if (octet & 0x80) == 0x00 {
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

                    for k in 1..width {
                        let octet = *(*parser).raw_buffer.pointer.add(k);
                        if (octet & 0xC0) != 0x80 {
                            return yaml_parser_set_reader_error(
                                parser,
                                b"invalid trailing UTF-8 octet\0".as_ptr() as *const c_char,
                                (*parser).offset + k,
                                octet as c_int,
                            );
                        }
                        value = (value << 6) + (octet & 0x3F) as u32;
                    }

                    if !((width == 1)
                        || (width == 2 && value >= 0x80)
                        || (width == 3 && value >= 0x800)
                        || (width == 4 && value >= 0x10000))
                    {
                        return yaml_parser_set_reader_error(
                            parser,
                            b"invalid length of a UTF-8 sequence\0".as_ptr() as *const c_char,
                            (*parser).offset,
                            -1,
                        );
                    }

                    if (value >= 0xD800 && value <= 0xDFFF) || value > 0x10FFFF {
                        return yaml_parser_set_reader_error(
                            parser,
                            b"invalid Unicode character\0".as_ptr() as *const c_char,
                            (*parser).offset,
                            value as c_int,
                        );
                    }
                }

                YAML_UTF16LE_ENCODING | YAML_UTF16BE_ENCODING => {
                    let low: usize =
                        if (*parser).encoding == YAML_UTF16LE_ENCODING { 0 } else { 1 };
                    let high: usize =
                        if (*parser).encoding == YAML_UTF16LE_ENCODING { 1 } else { 0 };

                    if raw_unread < 2 {
                        if (*parser).eof != 0 {
                            return yaml_parser_set_reader_error(
                                parser,
                                b"incomplete UTF-16 character\0".as_ptr() as *const c_char,
                                (*parser).offset,
                                -1,
                            );
                        }
                        incomplete = 1;
                        break;
                    }

                    value = (*(*parser).raw_buffer.pointer.add(low)) as u32
                        + ((*(*parser).raw_buffer.pointer.add(high)) as u32) << 8;

                    if (value & 0xFC00) == 0xDC00 {
                        return yaml_parser_set_reader_error(
                            parser,
                            b"unexpected low surrogate area\0".as_ptr() as *const c_char,
                            (*parser).offset,
                            value as c_int,
                        );
                    }

                    if (value & 0xFC00) == 0xD800 {
                        width = 4;

                        if raw_unread < 4 {
                            if (*parser).eof != 0 {
                                return yaml_parser_set_reader_error(
                                    parser,
                                    b"incomplete UTF-16 surrogate pair\0".as_ptr()
                                        as *const c_char,
                                    (*parser).offset,
                                    -1,
                                );
                            }
                            incomplete = 1;
                            break;
                        }

                        value2 = (*(*parser).raw_buffer.pointer.add(low + 2)) as u32
                            + ((*(*parser).raw_buffer.pointer.add(high + 2)) as u32) << 8;

                        if (value2 & 0xFC00) != 0xDC00 {
                            return yaml_parser_set_reader_error(
                                parser,
                                b"expected low surrogate area\0".as_ptr() as *const c_char,
                                (*parser).offset + 2,
                                value2 as c_int,
                            );
                        }

                        value = 0x10000 + ((value & 0x3FF) << 10) + (value2 & 0x3FF);
                    } else {
                        width = 2;
                    }
                }

                _ => {
                    debug_assert!(false, "Impossible");
                }
            }

            if incomplete != 0 {
                break;
            }

            // Check valid character range
            if !(value == 0x09
                || value == 0x0A
                || value == 0x0D
                || (value >= 0x20 && value <= 0x7E)
                || value == 0x85
                || (value >= 0xA0 && value <= 0xD7FF)
                || (value >= 0xE000 && value <= 0xFFFD)
                || (value >= 0x10000 && value <= 0x10FFFF))
            {
                return yaml_parser_set_reader_error(
                    parser,
                    b"control characters are not allowed\0".as_ptr() as *const c_char,
                    (*parser).offset,
                    value as c_int,
                );
            }

            // Advance raw buffer pointer
            (*parser).raw_buffer.pointer = (*parser).raw_buffer.pointer.add(width);
            (*parser).offset += width;

            // Encode value as UTF-8 into buffer
            if value <= 0x7F {
                *(*parser).buffer.last = value as u8;
                (*parser).buffer.last = (*parser).buffer.last.add(1);
            } else if value <= 0x7FF {
                *(*parser).buffer.last = (0xC0 + (value >> 6)) as u8;
                (*parser).buffer.last = (*parser).buffer.last.add(1);
                *(*parser).buffer.last = (0x80 + (value & 0x3F)) as u8;
                (*parser).buffer.last = (*parser).buffer.last.add(1);
            } else if value <= 0xFFFF {
                *(*parser).buffer.last = (0xE0 + (value >> 12)) as u8;
                (*parser).buffer.last = (*parser).buffer.last.add(1);
                *(*parser).buffer.last = (0x80 + ((value >> 6) & 0x3F)) as u8;
                (*parser).buffer.last = (*parser).buffer.last.add(1);
                *(*parser).buffer.last = (0x80 + (value & 0x3F)) as u8;
                (*parser).buffer.last = (*parser).buffer.last.add(1);
            } else {
                *(*parser).buffer.last = (0xF0 + (value >> 18)) as u8;
                (*parser).buffer.last = (*parser).buffer.last.add(1);
                *(*parser).buffer.last = (0x80 + ((value >> 12) & 0x3F)) as u8;
                (*parser).buffer.last = (*parser).buffer.last.add(1);
                *(*parser).buffer.last = (0x80 + ((value >> 6) & 0x3F)) as u8;
                (*parser).buffer.last = (*parser).buffer.last.add(1);
                *(*parser).buffer.last = (0x80 + (value & 0x3F)) as u8;
                (*parser).buffer.last = (*parser).buffer.last.add(1);
            }

            (*parser).unread += 1;
        }

        // On EOF, put NUL into buffer and return
        if (*parser).eof != 0 {
            *(*parser).buffer.last = 0;
            (*parser).buffer.last = (*parser).buffer.last.add(1);
            (*parser).unread += 1;
            return 1;
        }
    }

    if (*parser).offset >= MAX_FILE_SIZE {
        return yaml_parser_set_reader_error(
            parser,
            b"input is too long\0".as_ptr() as *const c_char,
            (*parser).offset,
            -1,
        );
    }

    1
}
