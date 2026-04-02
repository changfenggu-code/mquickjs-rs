set shell := ["powershell.exe", "-NoLogo", "-Command"]

default:
  @just --list

# Benchmark entrypoints
bench-canaries:
  sh ./benches/run_canaries.sh

bench-canaries-list:
  sh ./benches/run_canaries.sh --list

bench-compare:
  sh ./benches/compare.sh

bench-compare-exec:
  sh ./benches/compare.sh --execution-only

bench-compare-all:
  sh ./benches/compare.sh --all

bench-compare-all-exec:
  sh ./benches/compare.sh --all --execution-only

bench-report-all:
  sh ./benches/update_criterion_results.sh
  sh ./benches/compare.sh --execution-only
  sh ./benches/compare.sh

bench-report-all-full:
  sh ./benches/update_criterion_results.sh
  sh ./benches/compare.sh --all --execution-only
  sh ./benches/compare.sh --all

bench-compare-tier tier:
  sh ./benches/compare.sh --tier {{tier}}

bench-compare-tier-exec tier:
  sh ./benches/compare.sh --tier {{tier}} --execution-only

bench-compare-all-rust:
  sh ./benches/compare.sh --all --rust-only

bench-compare-list-tiers:
  sh ./benches/compare.sh --list-tiers

bench-runs runs:
  $env:BENCH_RUNS="{{runs}}"; sh ./benches/compare.sh

bench-exec-iters iters:
  $env:BENCH_EXEC_ITERS="{{iters}}"; sh ./benches/compare.sh --execution-only

# Criterion
bench-criterion:
  cargo bench --bench js_benchmarks

bench-criterion-report:
  sh ./benches/update_criterion_results.sh

bench-criterion-no-run:
  cargo bench --bench js_benchmarks --no-run

bench-criterion-one name:
  cargo bench --bench js_benchmarks -- "{{name}}"

# Build helpers
bench-build:
  cargo build --release --bin mqjs --bin bench_exec_helper

# Diagnostic bins
dump-bytecode case:
  cargo run --bin dump_bytecode -- {{case}}

profile-hotspots:
  cargo run --features dump --bin profile_hotspots

analyze-dense-array:
  cargo run --bin analyze_dense_array_layers

json-parse-probe:
  cargo run --bin json_parse_probe

gc-memory-probe:
  cargo run --bin gc_memory_probe

gc-overhead-probe:
  cargo run --bin gc_overhead_probe

deep-property-probe:
  cargo run --bin deep_property_probe

# Test / validation helpers
test-benchmark-workflow:
  cargo test --test benchmark_workflow_tests

fmt-check:
  cargo fmt --check

clippy:
  cargo clippy -p mquickjs-rs --all-targets -- -D warnings

test-engine:
  cargo test -p mquickjs-rs
