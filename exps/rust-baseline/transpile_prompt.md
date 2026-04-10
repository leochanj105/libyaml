You are transpiling a C library to Rust.

- Original C source: /home/leochanj/Desktop/libyaml/src/
- Public headers:    /home/leochanj/Desktop/libyaml/include/
- Config header:     /home/leochanj/Desktop/libyaml/build/include/config.h
- Output Rust project: the current working directory

The library is libyaml, a YAML 1.1 parser and emitter. Read ALL .c and .h files
in the source tree (api.c, scanner.c, parser.c, reader.c, writer.c, emitter.c,
loader.c, dumper.c, yaml_private.h, yaml.h) before starting.

Produce a complete, equivalent Rust library that:

1. Reproduces the exact behavior of the C code for all inputs.
2. Exposes every non-static C function as `#[no_mangle] pub extern "C"` using
   the **same name** as the C function. Example:
   ```rust
   #[no_mangle]
   pub extern "C" fn yaml_parser_initialize(parser: *mut yaml_parser_t) -> libc::c_int {
       // ... Rust implementation ...
   }
   ```
3. Builds with `cargo build` (produce Cargo.toml, src/lib.rs, and any module
   files you need). Write all files to the current directory.

## What "transpile" means

Re-implement every C function body in Rust with equivalent logic. Loops,
conditionals, arithmetic, pointer arithmetic, array indexing — all translated.

The following are NOT acceptable and will be rejected:
- Calling back into the original C library via `extern "C"` or `#[link(...)]`
- FFI bindings (e.g. bindgen output)
- `unimplemented!()`, `todo!()`, or stubs
- Delegating any logic to C via `unsafe extern "C" { fn yaml_...; }` blocks

The Rust library must be entirely self-contained. It must not reference or
link against the original C source at all.

## Rules

- Do NOT fix bugs in the C code — reproduce its behavior exactly.
- Preserve exact order of operations, error checks, and validation.
- If the C code uses global/static state, reproduce it faithfully.

## Cargo.toml

```toml
[package]
name = "libyaml_rs"
version = "0.1.0"
edition = "2021"

[lib]
name = "yaml"
crate-type = ["staticlib", "cdylib"]

[dependencies]
libc = "0.2"
```

Both crate types are required. Do NOT add `#![no_std]` — staticlib requires std.

