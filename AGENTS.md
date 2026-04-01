# AGENTS.md

This repository is an already-split Rust workspace with three practical areas:

- `mquickjs-rs` (root crate): the general-purpose JavaScript engine
- `led-runtime/` (workspace member): the LED/effect product runtime built on top of the engine
- `contrib/mquickjs/`: local C reference/upstream comparison tree

Use this file as the default guidance for any agent working in this repo.

## 1. Current Repository Reality

- The workspace split is already in effect today. Do not treat `led-runtime/` as a future plan.
- When documentation disagrees with the actual code, trust this order:
  1. `Cargo.toml` / workspace layout
  2. current source tree
  3. tests and CI workflows
  4. older planning/migration documents
- `contrib/mquickjs/` is the current checked-in C reference tree in this worktree.

## 2. Repository Structure

### Engine layer (root crate)

Treat the following as engine code:

- `src/`
  - especially `src/parser/`, `src/vm/`, `src/runtime/`, `src/gc/`, `src/context.rs`, `src/value.rs`
- `tests/`
  - especially `tests/eval_integration.rs`
  - `tests/error_messages.rs`
  - `tests/gc_tests.rs`
  - `tests/runtime_tests.rs`
  - `tests/stack_opcode_tests.rs`
  - `tests/value_tests.rs`
  - `tests/vm_tests.rs`
  - `tests/compiler_tests.rs`
  - `tests/lexer_tests.rs`
  - `tests/util_tests.rs`
- `benches/`
- `js/examples/`
- `js/tests/`
- `docs/HOW_IT_WORKS.md`
- `docs/BENCHMARK_ANALYSIS.md`
- `docs/BENCHMARK_ANALYSIS.zh.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`
- `docs/GC_PROGRESS.md`
- `docs/DUAL_MEMORY_ARCHITECTURE.md`
- `docs/PROJECT_STRUCTURE.md`
- `docs/JS_FEATURE_SPEC.md`
- `docs/WORKSPACE_SPLIT_PLAN.md`
- `docs/NUMERIC_AUDIT_TODO.md`
- `IMPLEMENTATION_PLAN.md`
- `IMPLEMENTATION_PLAN.zh.md`
- `README.md`
- `README.zh.md`

### Product layer (`led-runtime/`)

Treat the following as product/runtime code:

- `led-runtime/src/`
- `led-runtime/tests/`
- `led-runtime/tests/effect_api.rs`
- `led-runtime/tests/effects.rs`
- `led-runtime/examples/`
- `led-runtime/js/effects/`
- `led-runtime/docs/`
- `led-runtime/WORKLINE.md`

### Reference layer (`contrib/mquickjs/`)

Treat the following as upstream/reference material, not the Rust engine itself:

- `contrib/mquickjs/`

Use it for:

- behavioral comparison
- benchmark comparison
- parity checks against the original C implementation

Do not:

- move product-layer logic into `contrib/`
- treat `contrib/` layout as the desired Rust architecture
- make edits there unless the task explicitly involves the reference tree or benchmark harness

## 3. Default Working Rules

- Keep engine work and product work separated.
- Do not reintroduce product-layer APIs into the root engine crate unless explicitly requested.
- Prefer putting LED/effect-specific logic in `led-runtime/`, not the root crate.
- Prefer keeping engine changes general-purpose and reusable.
- Treat `no_std` as the default operating constraint for engine work unless the user explicitly asks for a `std`-only change.
- When changing engine code, prefer designs that remain compatible with `no_std`.
- When validating engine behavior, prefer `no_std`-compatible reasoning and tests; do not assume desktop-only facilities are acceptable by default.
- The root crate currently enables `std` by default, but that is a feature default, not the design target for engine changes.
- The active runtime GC path is the index-based mark-sweep collector in `src/vm/gc.rs` driven by `Interpreter::gc_collect()`.
- Do not assume `src/gc/collector.rs` is the primary runtime GC path; treat it as heap/allocator-layer or Plan C reference work unless the task explicitly targets that subsystem.
- `led-runtime` itself is `no_std`; keep that in mind when changing product/runtime code.
- `EffectEngine::from_source(...)` is gated behind the `led-runtime` `compiler` feature; bytecode-loading paths are the baseline production path.

## 4. API Layering

### Engine crate (`mquickjs-rs`)

The root crate should primarily expose or implement general engine primitives such as:

- `Context`
- `Value`
- `FunctionBytecode`
- parser/compiler infrastructure
- VM/runtime/builtin infrastructure

### Product crate (`led-runtime`)

The product/runtime crate owns:

- `EffectEngine`
- `EffectInstance`
- `EffectManager`
- `ConfigValue`
- effect-host/runtime orchestration

When in doubt:

- generic JS runtime feature -> root crate
- LED/effect host feature -> `led-runtime`
- C parity/reference comparison -> `contrib/mquickjs/`

## 5. Testing Guidance

Core rule:

- Every code change must be accompanied by corresponding tests whenever the change affects behavior, semantics, public API, bug fixes, or optimization-sensitive logic.
- Prefer adding regression tests that would fail before the change and pass after the change.
- Do not treat a code change as complete unless its corresponding regression coverage has been added or consciously justified as unnecessary.

### For engine changes

Prefer running:

- `cargo test -p mquickjs-rs`
- targeted tests when possible:
  - `cargo test -p mquickjs-rs --test eval_integration`
  - `cargo test -p mquickjs-rs --test error_messages`
  - `cargo test -p mquickjs-rs --test gc_tests`
  - `cargo test -p mquickjs-rs --test runtime_tests`
  - `cargo test -p mquickjs-rs --test stack_opcode_tests`
  - `cargo test -p mquickjs-rs --test vm_tests`
  - `cargo test -p mquickjs-rs --test compiler_tests`
  - `cargo test -p mquickjs-rs --test lexer_tests`
  - `cargo test -p mquickjs-rs --test util_tests`
  - `cargo test -p mquickjs-rs --test value_tests`

Additional engine validation:

- `cargo build -p mquickjs-rs --release --no-default-features`
- `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`

Additional rule:

- Engine changes must preserve `no_std` compatibility as the default requirement.
- Do not consider an engine change complete if it only works in `std` mode unless the task explicitly targets `std`.
- For engine fixes and optimizations, add or update regression tests that lock in the intended behavior or performance-sensitive semantics.
- For GC-related engine changes, prefer regression tests that exercise real runtime liveness paths rather than helper functions in isolation.
- For parser/compiler/VM work, prefer tests that execute through `Context` or real bytecode flow, not only isolated helper assertions.

### For product/runtime changes

Prefer running:

- `cargo test -p led-runtime`
- targeted tests when possible:
  - `cargo test -p led-runtime --test effect_api`
  - `cargo test -p led-runtime --test effects`

Additional product/runtime validation when relevant:

- `cargo build -p led-runtime`

Additional rule:

- Product/runtime changes should also add matching regression tests whenever behavior changes.
- Examples are useful, but examples do not replace regression tests.
- If a change affects host-facing effect lifecycle/configuration behavior, add or update `led-runtime/tests/effect_api.rs` or `led-runtime/tests/effects.rs`.

### For workspace-wide changes

Prefer running:

- `cargo test --workspace`
- `cargo fmt --check`

Use CI as the reference for broad validation expectations:

- `cargo clippy -- -D warnings`
- `cargo build --release`
- root JS example smoke tests through `./target/release/mqjs js/examples/*.js`

### For benchmark-related work

- local comparison script: `benches/compare.sh`
- CI benchmark workflow: `.github/workflows/bench.yml`
- benchmark interpretation: `docs/BENCHMARK_ANALYSIS.md`
- optimization backlog: `docs/ENGINE_OPTIMIZATION_TASKLIST.md`

Important current note:

- if you touch benchmark plumbing, update path assumptions consistently instead of changing only one place

Do not treat a benchmark change as complete unless the benchmark method and result interpretation remain consistent.

## 6. Documentation Rules

- Keep English/Chinese document pairs synchronized when one side is updated.
- When changing an English documentation file, update the corresponding Chinese documentation file in the same turn unless the user explicitly says not to.
- Keep the meaning aligned across both versions; do not let one side drift or become stale.
- All Chinese documentation files must be saved as UTF-8.
- When updating paired English/Chinese docs, ensure the Chinese version remains valid UTF-8 without mojibake.

Documentation ownership:

- Engine documentation belongs in root `docs/`.
- Product/runtime documentation belongs in `led-runtime/docs/`.
- Root README files describe the repository and engine/workspace shape.
- Product API behavior belongs in `led-runtime/docs/EFFECT_ENGINE_API.md`.

When these topics change, update:

- benchmark interpretation:
  - `docs/BENCHMARK_ANALYSIS.md`
  - `docs/BENCHMARK_ANALYSIS.zh.md`
- engine optimization priorities:
  - `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
  - `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`
- engine GC architecture/status:
  - `docs/GC_PROGRESS.md`
  - `docs/HOW_IT_WORKS.md` if the conceptual model changed
- product runtime API behavior:
  - `led-runtime/docs/EFFECT_ENGINE_API.md`
  - related product docs as needed
- repository structure or workspace boundaries:
  - `AGENTS.md`
  - `docs/PROJECT_STRUCTURE.md`
  - `docs/WORKSPACE_SPLIT_PLAN.md` if it still discusses the affected boundary
  - `README.md`
  - `README.zh.md`

## 7. Benchmark Priorities

The current engine optimization focus is:

1. benchmark baseline correctness
2. call-path hot path optimization
3. native/builtin argument marshalling optimization
4. dense array fast paths
5. memory measurement and GC improvements

For the current no_std-oriented engine path, prioritize benchmark coverage for:

- `method_chain`
- `runtime_string_pressure`
- `for_of_array`
- `deep_property`

Defer `RegExp`-focused benchmarks unless the task is explicitly std/RegExp-oriented.

## 8. no_std-First Policy

- The default assumption for this repository is that engine changes should work under `no_std`.
- `std` support is important, but it is not the default design target for new engine changes.
- If a tradeoff appears between a convenient `std`-only solution and a `no_std`-compatible solution, prefer the `no_std`-compatible solution unless explicitly instructed otherwise.
- When adding tests, benchmarks, examples, or helper paths, keep the distinction clear:
  - engine correctness and optimization work should remain `no_std`-aware
  - desktop tooling and product demos may use `std`, but should not drive engine-only design by default

## 9. Change Style

- Prefer root-cause fixes over cosmetic patches.
- Keep changes minimal and local to the correct layer.
- Do not silently change public semantics across both crates unless necessary.
- Preserve the current direction:
  - `EffectManager` is the preferred product-layer entry point
  - root crate remains engine-focused
- If a source file already contains mojibake or legacy transitional comments, avoid spreading that state further; fix encoding only when it is in scope and can be done safely.

## 10. When Unsure

If a requested change could belong to either layer, prefer this decision order:

1. Is it effect/LED/product-specific? -> `led-runtime`
2. Is it general JS engine/runtime behavior? -> root crate
3. Is it benchmark/optimization work for the engine? -> root crate
4. Is it upstream comparison/parity/reference material? -> `contrib/mquickjs/`

If the task is about "what is true right now", verify against the live codebase instead of assuming older docs are current.
