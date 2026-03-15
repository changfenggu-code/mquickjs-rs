# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **Note**: 中文版本 available at [CLAUDE.zh.md](CLAUDE.zh.md)

## Project Overview

MQuickJS-RS is a **Research Project** — a pure safe Rust port of Fabrice Bellard's [MQuickJS](https://github.com/bellard/mquickjs) minimalist JavaScript engine. It implements an ES5 subset with a tracing mark-compact GC, stack-based bytecode VM, and no `unsafe` code.

**Critical for ESP32/Bare Metal Development**: This project is designed to run on ESP32 bare metal with `no_std`. When adding features or modifying code, always verify `no_std` compatibility.

## Commands

```bash
# Build
cargo build
cargo build --release

# Build for ESP32 bare metal (no_std)
cargo build --release --no-default-features

# Run tests (458 tests)
cargo test

# Run a single test by name
cargo test test_name

# Run tests with stdout visible
cargo test -- --nocapture

# Lint (CI enforces zero warnings)
cargo clippy -- -D warnings

# Format check
cargo fmt --check
cargo fmt

# Run the JS REPL/CLI
cargo run --bin mqjs
cargo run --bin mqjs -- script.js
cargo run --bin mqjs -- -e "1 + 2"

# Benchmarks (requires C submodule for comparison)
cargo bench
git submodule update --init
./benches/compare.sh

# Build with debug features
cargo build --features dump        # enable bytecode/GC dumps
cargo build --features debug-gc    # GC moves objects on every allocation
```

## Architecture

The execution pipeline is: **Source JS → Lexer → Parser/Compiler → Bytecode → Interpreter (VM)**

### Key modules

- **[src/context.rs](src/context.rs)** — `Context` is the public API entry point. Owns a `Heap` (GC) and `Interpreter`. Call `ctx.eval(source)` to run JavaScript.

- **[src/value.rs](src/value.rs)** — `Value` tagged union: integers (31-bit inline), special values (null/undefined/bool), and pointer indices for strings, objects, arrays, closures, errors, regexps, typed arrays, and built-in objects. All heap references are indices into interpreter-owned `Vec`s, not raw pointers.

- **[src/parser/lexer.rs](src/parser/lexer.rs)** — Tokenizer.

- **[src/parser/compiler.rs](src/parser/compiler.rs)** — Recursive-descent parser + bytecode emitter (precedence climbing for expressions). Produces `FunctionBytecode`.

- **[src/vm/opcode.rs](src/vm/opcode.rs)** — ~80 opcodes.

- **[src/vm/interpreter.rs](src/vm/interpreter.rs)** — `Interpreter` struct, bytecode dispatch loop (`run`), and arithmetic/comparison operators. Owns all heap-allocated JS objects as `Vec`s indexed by `Value` tags.

- **[src/vm/property.rs](src/vm/property.rs)** — `get_*_property()` methods that dispatch property access for arrays, strings, numbers, errors, regexps, typed arrays, and built-in objects.

- **[src/vm/natives.rs](src/vm/natives.rs)** — ~100 native function implementations (Array/String/Math/JSON/RegExp/Date methods, `format_value`, JSON parser, etc.).

- **[src/vm/stack.rs](src/vm/stack.rs)** — Value stack with call frames.

- **[src/gc/](src/gc/)** — Arena allocator (`allocator.rs`) and mark-compact collector (`collector.rs`). The GC is mostly a stub; actual object lifetime is managed via the `Vec`-of-objects pattern in the interpreter.

- **[src/runtime/](src/runtime/)** — Object, string, array, function, and property table types used by the interpreter.

- **[src/builtins/](src/builtins/)** — Built-in method implementations (Array, String, Math, JSON, RegExp, Error, TypedArray, etc.).

- **[src/bin/mqjs.rs](src/bin/mqjs.rs)** — CLI/REPL using `rustyline`. Supports `-e`, `-i`, `-I`, `-d`, `-c`, `--memory-limit`.

### Value encoding

Heap object types are encoded using marker bits in the `Value` u64:
- `BUILTIN_OBJECT_MARKER` → built-in globals (Math, JSON, console, etc.) by `BUILTIN_*` index
- `ERROR_OBJECT_MARKER` (bit 20) → `ErrorObject` by index into `interpreter.error_objects`
- `REGEXP_OBJECT_MARKER` (bit 19) → `RegExpObject` by index
- `TYPED_ARRAY_MARKER` (bit 18) → `TypedArrayObject` by index

### Method dispatch

Object property/method access goes through `get_*_property()` helpers in the interpreter (e.g. `get_array_property`, `get_string_property`, `get_builtin_property`). The `GetField2` + `CallMethod` opcode pair is emitted for method call syntax (`obj.method()`), keeping `this` on the stack.

### Tests

Tests live inline in each source file (`#[cfg(test)]` modules). The 458 tests cover the full language feature set and all built-in methods.

## Workspace Structure

This is a Cargo workspace with two members:

- **mquickjs-rs** (root) — Core JavaScript engine (`src/`)
- **led-runtime** (`led-runtime/`) — LED effect runtime built on top of mquickjs-rs

## Conventions

- **Always verify after making changes** — After any code changes or running scripts, verify the output is as expected. Don't assume it works — test it.
- **Bilingual docs must stay in sync** — `IMPLEMENTATION_PLAN.md` (English) and `IMPLEMENTATION_PLAN.zh.md` (Chinese) must always be updated together. When editing one, apply the equivalent change to the other in the same session. Similarly for `CLAUDE.md` / `CLAUDE.zh.md`.
- **CI must pass** — run `cargo clippy -- -D warnings` and `cargo fmt --check` before considering work done.
- **JS scripts** live in `js/examples/` (feature demos) and `js/tests/` (error handling tests). The top-level `examples/` is reserved for Rust examples by Cargo convention.
- New opcodes go in [src/vm/opcode.rs](src/vm/opcode.rs), with the handler added in the interpreter's main `match` in [src/vm/interpreter.rs](src/vm/interpreter.rs).
- New built-in methods go in `src/builtins/<object>.rs`, wired up via `get_*_property()` in the interpreter.
- **`no_std` is required for ESP32** — The project must compile without `std`. When adding dependencies, verify they have `no_std` support or add `default-features = ["std"]` to `Cargo.toml` and use `#[cfg(feature = "std")]` gates.
- **Memory-constrained design** — ESP32 has limited RAM (typically 320-520KB). Prefer inline allocation, avoid dynamic allocation in hot paths, and use tagged values for integers to minimize heap usage.
- **Cross-compilation target** — For ESP32, use target: `riscv32imac-unknown-none-elf`. Run `rustup target add riscv32imac-unknown-none-elf` if not already installed.
