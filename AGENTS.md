# AGENTS.md

This repository is a split workspace with two distinct layers:

- `mquickjs-rs` (root crate): the JavaScript engine itself
- `led-runtime/` (workspace member): the LED/effect product runtime built on top of the engine

Use this file as the default guidance for any agent working in this repo.

## 1. Repository Structure

### Engine layer (root crate)

Treat the following as engine code:

- `src/`
  - especially `src/parser/`, `src/vm/`, `src/runtime/`, `src/gc/`, `src/context.rs`, `src/value.rs`
- `tests/eval_integration.rs`
- `tests/error_messages.rs`
- `benches/`
- `docs/HOW_IT_WORKS.md`
- `docs/BENCHMARK_ANALYSIS.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`
- `docs/JS_FEATURE_SPEC.md`
- `docs/WORKSPACE_SPLIT_PLAN.md`
- `IMPLEMENTATION_PLAN.md`
- `IMPLEMENTATION_PLAN.zh.md`

### Product layer (`led-runtime/`)

Treat the following as product/runtime code:

- `led-runtime/src/`
- `led-runtime/tests/`
- `led-runtime/examples/`
- `led-runtime/js/effects/`
- `led-runtime/docs/`
- `led-runtime/WORKLINE.md`

## 2. Default Working Rules

- Keep engine work and product work separated.
- Do not reintroduce product-layer APIs into the root engine crate unless explicitly requested.
- Prefer putting LED/effect-specific logic in `led-runtime/`, not the root crate.
- Prefer keeping engine changes general-purpose and reusable.
- Treat `no_std` as the default operating constraint for engine work unless the user explicitly asks for a `std`-only change.
- When changing engine code, prefer designs that remain compatible with `no_std`.
- When validating engine behavior, prefer `no_std`-compatible reasoning and tests; do not assume desktop-only facilities are acceptable by default.

## 3. API Layering

### Engine crate (`mquickjs-rs`)

The root crate should primarily expose general engine primitives such as:

- `Context`
- `Value`
- `FunctionBytecode`
- VM/runtime/parser infrastructure

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

## 4. Testing Guidance

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

Additional rule:

- Engine changes must preserve `no_std` compatibility as the default requirement.
- Do not consider an engine change complete if it only works in `std` mode unless the task explicitly targets `std`.
- For engine fixes and optimizations, add or update regression tests that lock in the intended behavior or performance-sensitive semantics.

### For product/runtime changes

Prefer running:

- `cargo test -p led-runtime`
- targeted tests when possible:
  - `cargo test -p led-runtime --test effect_api`
  - `cargo test -p led-runtime --test effects`

Additional rule:

- Product/runtime changes should also add matching regression tests whenever behavior changes.
- Examples are useful, but examples do not replace regression tests.

### For benchmark-related work

- local comparison script: `benches/compare.sh`
- CI benchmark workflow: `.github/workflows/bench.yml`
- benchmark interpretation: `docs/BENCHMARK_ANALYSIS.md`

Do not treat a benchmark change as complete unless the benchmark method and result interpretation remain consistent.

## 5. Documentation Rules

- Engine documentation belongs in root `docs/`.
- Product/runtime documentation belongs in `led-runtime/docs/`.
- If a change affects benchmark interpretation, update `docs/BENCHMARK_ANALYSIS.md`.
- If a change affects engine optimization priorities, update:
  - `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
  - `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`
- If a change affects product runtime API behavior, update:
  - `led-runtime/docs/EFFECT_ENGINE_API.md`
  - and related product docs as needed.

## 6. Benchmark Priorities

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

## 7. no_std First Policy

- The default assumption for this repository is that engine changes should work under `no_std`.
- `std` support is important, but it is not the default design target for new engine changes.
- If a tradeoff appears between a convenient `std`-only solution and a `no_std`-compatible solution, prefer the `no_std`-compatible solution unless explicitly instructed otherwise.
- When adding tests, benchmarks, or helper paths, keep the distinction clear:
  - engine correctness and optimization work should remain `no_std`-aware
  - desktop tooling and product demos may use `std`, but should not drive engine-only design by default

## 8. Change Style

- Prefer root-cause fixes over cosmetic patches.
- Keep changes minimal and local to the correct layer.
- Do not silently change public semantics across both crates unless necessary.
- Preserve the current direction:
  - `EffectManager` is the preferred product-layer entry point
  - root crate remains engine-focused

## 9. When Unsure

If a requested change could belong to either layer, prefer this decision order:

1. Is it effect/LED/product-specific? -> `led-runtime`
2. Is it general JS engine/runtime behavior? -> root crate
3. Is it benchmark/optimization work for the engine? -> root crate
