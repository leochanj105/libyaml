use crate::*;
use libc::{c_char, c_int, size_t};

unsafe fn yaml_emitter_set_writer_error(
    emitter: *mut yaml_emitter_t,
    problem: *const c_char,
) -> c_int {
    (*emitter).error = YAML_WRITER_ERROR;
    (*emitter).problem = problem;
    0
}

#[no_mangle]
pub unsafe extern "C" fn yaml_emitter_flush(emitter: *mut yaml_emitter_t) -> c_int {
    debug_assert!(!emitter.is_null());
    debug_assert!((*emitter).write_handler.is_some());
    debug_assert!((*emitter).encoding != YAML_ANY_ENCODING);

    (*emitter).buffer.last = (*emitter).buffer.pointer;
    (*emitter).buffer.pointer = (*emitter).buffer.start;

    // Check if buffer is empty
    if (*emitter).buffer.start == (*emitter).buffer.last {
        return 1;
    }

    // If UTF-8, write directly
    if (*emitter).encoding == YAML_UTF8_ENCODING {
        let len = (*emitter).buffer.last.offset_from((*emitter).buffer.start) as usize;
        if ((*emitter).write_handler.unwrap())(
            (*emitter).write_handler_data,
            (*emitter).buffer.start,
            len,
        ) != 0
        {
            (*emitter).buffer.last = (*emitter).buffer.start;
            (*emitter).buffer.pointer = (*emitter).buffer.start;
            return 1;
        } else {
            return yaml_emitter_set_writer_error(
                emitter,
                b"write error\0".as_ptr() as *const c_char,
            );
        }
    }

    // Re-encode to UTF-16
    let low: usize = if (*emitter).encoding == YAML_UTF16LE_ENCODING { 0 } else { 1 };
    let high: usize = if (*emitter).encoding == YAML_UTF16LE_ENCODING { 1 } else { 0 };

    while (*emitter).buffer.pointer != (*emitter).buffer.last {
        let octet = *(*emitter).buffer.pointer;

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

        for k in 1..width {
            let octet = *(*emitter).buffer.pointer.add(k);
            value = (value << 6) + (octet & 0x3F) as u32;
        }

        (*emitter).buffer.pointer = (*emitter).buffer.pointer.add(width);

        if value < 0x10000 {
            *(*emitter).raw_buffer.last.add(high) = (value >> 8) as u8;
            *(*emitter).raw_buffer.last.add(low) = (value & 0xFF) as u8;
            (*emitter).raw_buffer.last = (*emitter).raw_buffer.last.add(2);
        } else {
            // Surrogate pair
            let value = value - 0x10000;
            *(*emitter).raw_buffer.last.add(high) = (0xD8 + (value >> 18)) as u8;
            *(*emitter).raw_buffer.last.add(low) = ((value >> 10) & 0xFF) as u8;
            *(*emitter).raw_buffer.last.add(high + 2) = (0xDC + ((value >> 8) & 0xFF)) as u8;
            *(*emitter).raw_buffer.last.add(low + 2) = (value & 0xFF) as u8;
            (*emitter).raw_buffer.last = (*emitter).raw_buffer.last.add(4);
        }
    }

    // Write the raw buffer
    let raw_len = (*emitter)
        .raw_buffer
        .last
        .offset_from((*emitter).raw_buffer.start) as usize;
    if ((*emitter).write_handler.unwrap())(
        (*emitter).write_handler_data,
        (*emitter).raw_buffer.start,
        raw_len,
    ) != 0
    {
        (*emitter).buffer.last = (*emitter).buffer.start;
        (*emitter).buffer.pointer = (*emitter).buffer.start;
        (*emitter).raw_buffer.last = (*emitter).raw_buffer.start;
        (*emitter).raw_buffer.pointer = (*emitter).raw_buffer.start;
        1
    } else {
        yaml_emitter_set_writer_error(emitter, b"write error\0".as_ptr() as *const c_char)
    }
}
