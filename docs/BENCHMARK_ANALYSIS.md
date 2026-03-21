# Benchmark Analysis

Chinese version: `docs/BENCHMARK_ANALYSIS.zh.md`

Related optimization backlogs:
- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

## Purpose

This document defines the current benchmark baseline and interpretation rules for the
`mquickjs-rs` engine.

It is engine-only documentation and does not cover `led-runtime` product-layer behavior.

## Baseline Policy

Benchmark analysis uses three complementary sources.

### 1. Local Criterion (`cargo bench`)

Primary use:

- precise Rust-only execution-phase timing analysis
- before/after optimization validation
- hotspot confirmation

This is the preferred source when deciding whether an engine optimization actually helped.

As of March 17, 2026, the Criterion harness compiles each benchmark script once and then
measures execution on fresh contexts. This intentionally reduces parser/compiler noise in
runtime optimization work.

### 2. Local Rust-vs-C comparison (`benches/compare.sh` or equivalent local comparison)

Primary use:

- compare current Rust engine performance against the C `mqjs` implementation
- estimate how far a path still is from the C implementation

This is the preferred source when evaluating cross-implementation distance.

### 3. CI benchmark summary (`.github/workflows/bench.yml`)

Primary use:

- trend tracking in GitHub Actions
- quick regression visibility on pushes and PRs
- GitHub-visible summary tables

The CI summary now provides:

- a Rust-vs-C comparison table
- a Rust-only Criterion table
- a startup baseline row

CI results are useful and visible on GitHub, but local Criterion and local Rust-vs-C
comparison remain the main sources for detailed investigation.

## Canonical Benchmark Set

The current engine baseline set is:

- legacy core set:
  - `fib`
  - `loop`
  - `array`
  - `json`
  - `sieve`
- first-wave expansion set:
  - `method_chain`
  - `runtime_string_pressure`
  - `for_of_array`
  - `deep_property`
- current secondary tracked benchmark:
  - `switch_case`

Deferred for the current `no_std`-first path:

- `regexp_test`
- `regexp_exec`

Future secondary additions:

- `switch_case`
- `for_in_object`

Currently tracked secondary benchmarks:

- `switch_case`
- `try_catch`
- `for_in_object`

## Current Baseline (Local Criterion, Rust-only)

Important current status on March 17, 2026:

- the Criterion harness was updated to compile once and benchmark execution on fresh contexts
- a full local Criterion revalidation was run for the primary benchmark set on the current worktree
- several values previously recorded on March 16, 2026 no longer match the current head
- older March 16 Criterion numbers are therefore not directly comparable to the current harness
- benchmark baseline cleanup should therefore be treated as active work again, not as closed work

### Historical Revalidated Snapshot (2026-03-17)

These values came from the first fresh local rerun after the compile-once
execution-focused Criterion harness landed. They are still useful as a historical
reference point, but they should no longer be treated as the current trusted
primary-set snapshot for the current head.

| Benchmark | Current local snapshot |
|-----------|------------------------|
| `fib_iter 1k` | `2.330–2.379 ms` |
| `loop 10k` | `0.472–0.485 ms` |
| `array push 10k` | `0.491–0.502 ms` |
| `json parse 1k` | `0.736–0.754 ms` |
| `sieve 10k` | `2.069–2.103 ms` |
| `method_chain 5k` | `0.585–0.600 ms` |
| `runtime_string_pressure 4k` | `0.899–0.915 ms` |
| `for_of_array 20k` | `1.796–1.959 ms` |
| `deep_property 200k` | `14.925–15.235 ms` |

### Latest Broad Current-Head Rerun (2026-03-21)

This rerun was made after additional optimization work and cleanup attempts across
several hotspot areas. The important conclusion is not that these numbers are the
new “golden” baseline, but that the current head now diverges broadly from the
earlier 2026-03-17 snapshot and benchmark baseline correctness must be treated as
reopened work again.

| Benchmark | Latest broad rerun |
|-----------|--------------------|
| `array push 10k` | `766.00–946.17 µs` |
| `string concat 1k` | `164.20–205.55 µs` |
| `json parse 1k` | `1.8986–2.3272 ms` |
| `sieve 10k` | `2.3860–2.8523 ms` |
| `method_chain 5k` | `1.4008–1.7708 ms` |
| `runtime_string_pressure 4k` | `1.4943–1.8702 ms` |
| `for_of_array 20k` | `2.1365–2.5288 ms` |
| `deep_property 200k` | `19.605–23.419 ms` |

Current interpretation:

- this is no longer a “pick the next micro-hotspot” situation;
- the broad current-head rerun shows that several headline benchmarks have drifted
  substantially away from the earlier March 17 tier;
- because of that, benchmark baseline correctness should be considered reopened,
  and future optimization work should be compared against a freshly revalidated
  baseline rather than the older March 17 snapshot alone.

### Secondary Tracked Benchmarks

These values are useful for context, but they are now split into:

- historical March 17 snapshots
- newer March 21 reruns on the current stable worktree

This makes it explicit which secondary numbers are historical and which ones have
actually been rerun against the current head.

| Benchmark | Historical snapshot | Latest current-head rerun |
|-----------|---------------------|---------------------------|
| `switch 1k` | `0.132–0.136 ms` | `0.282–0.337 ms` |
| `try_catch 5k` | `0.341–0.349 ms` | `0.433–0.544 ms` |
| `for_in_object 20x2000` | `3.743–3.804 ms` | `9.911–11.992 ms` |

## Current Baseline (Local Rust vs C, process-inclusive)

These values are based on local repeated process execution averages and are useful for
cross-implementation comparison. They include process startup cost.

### Startup baseline (latest local rerun, 2026-03-21)

| Case | Rust | C | Ratio |
|------|------|---|-------|
| `mqjs -e "0"` | `41.300 ms` | `51.700 ms` | `0.799x` |

### Script comparisons (latest local rerun, 2026-03-21)

| Benchmark | Rust | C | Ratio | Notes |
|-----------|------|---|-------|-------|
| `fib` | `63.200 ms` | `49.900 ms` | `1.266x` | C faster |
| `loop` | `38.900 ms` | `34.800 ms` | `1.119x` | C faster |
| `array` | `35.300 ms` | `31.100 ms` | `1.135x` | C faster |
| `json` | `51.600 ms` | `44.500 ms` | `1.159x` | C faster |
| `sieve` | `55.500 ms` | `47.600 ms` | `1.166x` | C faster |
| `method_chain` | `31.700 ms` | `26.700 ms` | `1.191x` | C faster |
| `runtime_string_pressure` | `29.700 ms` | `25.500 ms` | `1.162x` | C faster |
| `for_of_array` | `39.900 ms` | `47.600 ms` | `0.840x` | Rust faster |
| `deep_property` | `53.500 ms` | `72.400 ms` | `0.739x` | Rust faster |
| `switch_case` | `72.300 ms` | `43.400 ms` | `1.666x` | C faster |
| `try_catch` | `41.900 ms` | `61.300 ms` | `0.684x` | Rust faster |
| `for_in_object` | `58.200 ms` | `64.900 ms` | `0.897x` | Rust faster |

## Interpretation Notes

### Strongest current signals

- `json parse` remains the most clearly active current-hotspot line because it did
  receive stable targeted wins, but it is also the clearest example of why the
  broad current-head benchmark set must be recalibrated before interpreting the
  next round of micro-optimizations.
- the latest local Rust-vs-C comparison no longer shows `json` as a Rust relative
  strength on the current head; in the newest process-inclusive rerun it is back
  on the C-faster side.
- `for_of_array`, `deep_property`, and `try_catch` now show Rust faster than C in
  the latest local process-inclusive comparison, while most of the rest of the
  current suite still favors C.
- `array push`, `sieve`, `for_of_array`, `deep_property`, `method_chain`, and
  `runtime_string_pressure` all still matter, but the latest broad rerun shows
  enough drift that older “healthy” labels should now be treated as provisional
  until baseline revalidation is complete.
- the string-path work already delivered real targeted wins in earlier rounds, but
  those earlier sub-millisecond interpretations should not be reused blindly
  against the current head without a fresh baseline.
- `for_of_array` and `deep_property` are still useful mechanism-specific windows,
  but their earlier best-tier numbers should now be read as historical snapshots,
  not as current guaranteed expectations.
- the secondary control-flow and exception benchmarks (`switch_case`, `try_catch`)
  also drifted materially upward on the latest current-head rerun, reinforcing the
  conclusion that baseline cleanup is still open work.
- `for_in_object` is now in a mixed state:
  - a structural key-reuse fix means the benchmark now completes on the Rust side
    instead of exhausting the runtime string table;
  - local process-level Rust-vs-C now also favors Rust on this path;
  - but the current Criterion timing remains far above its earlier historical tier,
    so this line should still currently be interpreted as “correctness fix landed,
    performance baseline still pending cleanup”.
- `switch_case` is now tracked as a secondary control-flow benchmark and has a new
  dedicated structural win on the current head via `SwitchCaseI8`, improving the
  benchmark into the `223–277 µs` range; however, local Rust-vs-C comparison still
  shows C ahead on this path (`1.666x`), so it should be treated as an isolated
  shape win inside the still-reopened baseline context.
- `try_catch` is now tracked as a secondary exception-control benchmark and has a recorded post-cleanup baseline.
- `for_in_object` is now tracked as a secondary iterator/control-flow benchmark with a first recorded baseline after iterator setup cleanup.
- after stage-closing the latest `json parse`, `switch_case`, and `for_in_object`
  rounds under the repository's practical-stop rule, the most plausible next shared
  structural target is no longer parser internals or secondary control-flow shapes;
  it is the `loop` / `sieve` comparison-and-branch skeleton.
- the reason is simple:
  - both `loop` and `sieve` still lag C in the latest local Rust-vs-C rerun;
  - both are broad headline benchmarks, not narrow secondary diagnostics;
  - and this path can be attacked without reopening the already-frozen dense-array
    read-side micro-pass.
- so the current recommendation after baseline cleanup is:
  - freeze `json parse`, `switch_case`, and `for_in_object` for now;
  - treat `loop` / `sieve` comparison-and-branch tightening as the next likely
    high-ROI structural mainline.
- a later targeted rerun refined that recommendation further:
  - `fib_iter 1k`: `5.3292–6.2708 ms`
  - `switch 1k`: `281.10–345.33 µs`
- interpretation:
  - `switch_case` still benefits from `SwitchCaseI8`, and its current bytecode shape is behaving as intended;
  - but `fib_iter` is now the more severe regression signal, because it has moved much farther away from its earlier `2.330–2.379 ms` historical tier than `switch 1k` has moved away from the current-head `~0.28–0.34 ms` class.
- so the current practical priority order should be read as:
  - first: call / recursion / small-loop overhead (`fib_iter`)
  - then: `loop` / `sieve` comparison-and-branch skeleton
  - while `switch_case` stays frozen unless a new switch-specific hotspot shape appears.
- dump-mode hotspot probing now backs up that priority order more directly:
  - `fib_iter`
    - still spends most of its time in call-adjacent loop machinery and local-slot traffic;
    - the iterative inner `fib` body is currently dominated by `GetLoc3`, `Drop`, `Dup`, `GetLoc0`, `Lte`, `GetLoc2`, `PutLoc2`, `Goto`, `Add`, `GetLoc4`, `PutLoc3`, `GetLoc8`, `PutLoc8`, and `IncLoc4Drop`;
    - this is consistent with a “call / recursion overhead plus small-loop/local-update overhead” interpretation rather than a pure arithmetic issue.
  - `switch_case`
    - `SwitchCaseI8` is clearly active in the latest dump-mode snapshot (`108000` executions);
    - remaining cost is therefore better explained by loop/add/update scaffolding around the switch chain, not by the old integer-case compare ladder itself.
- practical takeaway:
  - `fib_iter` is now the clearer next optimization target;
  - `switch_case` should remain frozen unless a new switch-specific hotspot shape appears.
- a first stable `fib_iter`-line win has now landed:
  - automatic GC trigger bookkeeping was removed from the generic JS `Call` / `CallMethod` / `CallConstructor` setup path;
  - the same bookkeeping is now charged at real GC-managed allocation sites instead.
- result:
  - targeted rerun: `fib_iter 1k` improved to `3.5469–4.1842 ms`
  - follow-up rerun stayed in the same class: `3.5909–4.2369 ms`
- interpretation:
  - this strongly suggests the previous `fib_iter` regression was not “just arithmetic got slower”, but that GC trigger accounting on every JS call had become part of the hot path;
  - the corresponding GC auto-trigger regression still passes, so this looks like a genuine structural win rather than a benchmark-only tradeoff.
- a later stable-tree rerun now shows:
  - `fib_iter 1k`: `3.0507–3.6993 ms`
  - `loop 10k`: `690.21–846.31 µs`
  - `sieve 10k`: `2.9538–3.5064 ms`
- interpretation update:
  - the retained `fib_iter` fix is still real on the current tree;
  - the next attempted `fib_iter` follow-up was reverted because it hurt `loop`, which means this line has already reached a practical stop point for the current phase;
  - the next structural priority should therefore move back to `loop` / `sieve`, while treating `fib_iter` as phase-closed unless a materially new hotspot shape appears.
- after returning to that `loop` / `sieve` mainline, a first shared structural win has now landed:
  - statement-position local arithmetic stores immediately after non-string `Add` / `Mul` now get consumed directly when the next opcode is `PutLoc0..4` or `PutLoc8 <idx>`.
- result:
  - `fib_iter 1k`: `2.2286–2.6849 ms`
  - `loop 10k`: `455.86–559.99 µs`
  - `sieve 10k`: `1.8323–2.1708 ms`
- interpretation:
  - this confirms the next useful shared target really was local arithmetic result materialization, not another `fib_iter`-specific call-path micro-pass;
  - it is also the first post-reprioritization win that clearly helps all three nearby workloads at once.

### Next Mainline Recommendation

With `fib_iter` and the current `loop` / `sieve` pass both now holding stable
wins for the current phase, the best current reopened headline candidate is:

1. `method_chain`

Current rationale:

- it remains a headline benchmark rather than only a secondary diagnostic;
- it still trails the C engine in the latest local Rust-vs-C table;
- and it remains materially above its earlier best execution-focused local tier,
  while the just-closed `fib_iter` and `loop` / `sieve` lines are now in much
  healthier ranges.

### Important caution

- Criterion numbers and process-inclusive Rust-vs-C numbers answer different questions.
- Short-running script comparisons can be more sensitive to startup overhead.
- Use Criterion first when validating an optimization inside the Rust engine.

## Already Completed Optimization Work Reflected In The Codebase

The current codebase still contains the first completed optimization rounds for:

- `deep_property`
  - small-object property lookup fast path
  - unified `GetField` / `GetField2` property dispatch
- `method_chain`
  - removed per-element temporary `Vec<Value>` allocation in array higher-order builtins
  - added `CallMethod` native small-argument fast path
  - added direct `Array.prototype.push` native fast path with an `argc == 1` shortcut
  - changed array `.push` property lookup to use the cached native index directly
  - replaced full-array cloning in higher-order array builtins with length-snapshot iteration plus live element reads
  - added dedicated `CallArrayMap1` / `CallArrayFilter1` / `CallArrayReduce2` opcodes
  - added dedicated `CallArrayPush1` for the dominant single-argument array-construction shape
- `runtime_string_pressure`
  - builds concat results in a single output buffer instead of materializing both operands separately
  - adds a narrower `string + int` / `int + string` fast path for decimal loop-index concatenation
  - adds bytecode-level `AddConstStringLeft` / `AddConstStringRight` specializations for compile-time string fragments in concat chains
  - folds adjacent compile-time string literals and lowers `const + value + const` into a dedicated `AddConstStringSurround` shape
  - adds `AppendConstStringToLoc0` plus a per-frame builder for the hottest statement-form local self-concat loop
  - adds `AddConstStringSurroundValue` for the common `const + value + const + value` concat-chain shape
  - adds a minimal deferred `RuntimeString` wrapper and serves `.length` from cached runtime-string lengths without forced flattening
- `for_of_array`
  - removed full array cloning from `ForOfStart`
  - added `ForOfNext` branch fusion for the common `IfTrue` loop-exit shape
- `loop` / `sieve`
  - added a `Dup + PutLocX + Drop` peephole fast path for common statement-update patterns
  - added `Lt/Lte` + `IfFalse/IfTrue` branch fusion

These changes are tracked in:

- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

Important current interpretation:

- these implementation changes are real and still present in the code
- the benchmark harness itself changed on March 17, 2026, so older Criterion numbers and current Criterion numbers must be treated as different measurement generations
- treat the table above as the current trusted execution-focused snapshot

## What "9.1.1 Benchmark Baseline Cleanup" Means Now

This baseline cleanup task is considered complete when:

- benchmark roles are clearly separated:
  - local Criterion -> precise Rust-only analysis
  - local Rust-vs-C comparison -> cross-implementation comparison
  - CI summary -> trend tracking + GitHub-visible analysis table
- the canonical benchmark set is defined
- the current baseline tables are recorded in one place
- future optimization rounds can use this document as the baseline reference

On March 17, 2026, this task had to be treated as reopened because both the recorded baseline
and the benchmark method itself changed. This document is therefore both the current baseline
reference and the place where that methodology change is now recorded explicitly.

## Runtime String Source Findings

Current dump-mode probing shows:

- `runtime_string_pressure` is almost entirely concat-driven.
- `for_in_object` is dominated by repeated `for-in` key handling and `GetLength` on the
  iterated key values.
- deep_property produces essentially no runtime strings.
- json_parse currently lands mostly in the generic other bucket.


Embedded note:
- Runtime string budget limits are intentionally deferred to later device integration work instead of being hard-coded into the engine at this stage.


Updated 9.3.3 findings:
- `runtime_string_pressure` is still entirely concat-driven.
- `for_in_object` no longer exhausts the runtime string table on the current stable
  worktree because repeated key strings are now reused on the `for-in` path.
- object_keys is now confirmed as a distinct runtime string source bucket.
- json_parse still lands in the generic other bucket and should be split further if we continue this line.
