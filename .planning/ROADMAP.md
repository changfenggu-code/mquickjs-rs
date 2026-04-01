# Roadmap: mquickjs-rs

**Created:** 2026-04-01
**Granularity:** standard
**Mode:** interactive
**All v1 requirements covered:** yes

## Overview

This roadmap treats the repository as a brownfield runtime core. The sequence intentionally locks down measurement first, closes the GC/resource story early, and only then broadens structural and optimization work so that benchmark wins are believable and maintainable.

| # | Phase | Goal | Requirements |
|---|-------|------|--------------|
| 1 | Benchmark Baseline Integrity | Make optimization measurements trustworthy and repeatable | BNCH-01, BNCH-02 |
| 2 | Complete GC And Memory Closure | Establish one active GC story with truthful runtime memory semantics | GC-01, GC-02, GC-03, GC-04, SAFE-02 |
| 3 | Runtime Safety Boundaries | Eliminate major lifetime and host-capability footguns around long-lived execution | SAFE-01, SAFE-03 |
| 4 | Structural Refactor Of Hotspots | Split monolithic hotspot areas into maintainable subsystem boundaries | ARCH-01, ARCH-02, ARCH-03 |
| 5 | Benchmark-Driven Optimization Rounds | Deliver measured speedups on canonical canary workloads without regressions | BNCH-03 |
| 6 | Product Runtime Alignment And Documentation | Reconcile host API expectations and documentation with the stabilized engine core | ARCH-04, HOST-01, HOST-02 |

## Phase Details

### Phase 1: Benchmark Baseline Integrity

**Goal:** Ensure local and CI performance data mean the same thing before further optimization work.

**Requirements:** BNCH-01, BNCH-02

**Plans:** 2 plans

Plans:
- [ ] `01-01-PLAN.md` - unify canonical Rust-vs-C benchmark plumbing across local and CI with regression validation
- [ ] `01-02-PLAN.md` - add canary rerun helper and synchronized benchmark workflow documentation

**Success criteria:**
1. Local Rust-vs-C benchmark workflow and CI benchmark workflow point at the same reference-tree location and assumptions
2. Canonical canary benchmarks are documented and easy to rerun after hotspot changes
3. Benchmark docs, scripts, and CI no longer disagree about the reference binary path or workflow

### Phase 2: Complete GC And Memory Closure

**Goal:** Make the runtime GC story complete enough that embedded/resource claims are technically defensible.

**Requirements:** GC-01, GC-02, GC-03, GC-04, SAFE-02

**Success criteria:**
1. One active GC path is clearly documented and reflected in code ownership boundaries
2. Runtime-managed containers are reclaimed/accounted in a way that matches actual execution behavior
3. Memory-limit and memory-stats semantics are either enforced or explicitly corrected so they are not misleading
4. Runtime-string lifetime/accumulation is addressed as part of the active memory model

### Phase 3: Runtime Safety Boundaries

**Goal:** Remove major execution-time footguns that undermine host confidence.

**Requirements:** SAFE-01, SAFE-03

**Success criteria:**
1. Bytecode/closure lifetime rules are made safe by API or explicit ownership discipline
2. Desktop-only host capabilities are clearer and less likely to leak into embedded assumptions
3. Long-lived execution scenarios have clearer safe usage patterns for embedders

### Phase 4: Structural Refactor Of Hotspots

**Goal:** Make the engine easier to optimize by decomposing the highest-risk monoliths without changing behavior.

**Requirements:** ARCH-01, ARCH-02, ARCH-03

**Success criteria:**
1. `src/vm/interpreter.rs` is split along real subsystem seams such as call paths, GC hooks, or object/iterator helpers
2. `src/vm/natives.rs` is broken into smaller builtin-family modules or an equivalent maintainable organization
3. Broad integration coverage is easier to target by subsystem, reducing the cost of future hotspot work

### Phase 5: Benchmark-Driven Optimization Rounds

**Goal:** Land measured wins on the chosen canary workloads after the baseline and structure are stable enough to trust them.

**Requirements:** BNCH-03

**Success criteria:**
1. At least one optimization round lands with clear before/after numbers on canonical workloads
2. Optimization claims are accompanied by targeted tests and documentation notes
3. Structural cleanup and optimization deltas remain attributable rather than mixed beyond explanation

### Phase 6: Product Runtime Alignment And Documentation

**Goal:** Ensure the product/runtime layer and repo docs reflect the stabilized engine direction.

**Requirements:** ARCH-04, HOST-01, HOST-02

**Success criteria:**
1. Core docs describe the actual workspace layout, active GC path, and runtime behavior
2. Product runtime APIs remain aligned with bytecode-first, no_std-aware host usage
3. Effect-host guidance and engine guidance no longer drift apart on ownership/resource assumptions

## Execution Notes

- Phase 1 and Phase 2 are the highest leverage because they decide whether later performance work is trustworthy.
- Phase 4 should avoid semantic change whenever possible; it is an enabling refactor phase.
- Phase 5 should focus on canary workloads already called out by the repo:
  - `method_chain`
  - `runtime_string_pressure`
  - `for_of_array`
  - `deep_property`

---
*Last updated: 2026-04-01 after initial roadmap creation*

