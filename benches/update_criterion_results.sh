#!/bin/bash
# Update the Rust Criterion section in benches/LATEST_RESULTS.md.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
REPORT_PATH="${BENCH_REPORT_PATH:-$SCRIPT_DIR/LATEST_RESULTS.md}"
TMP_DIR="$PROJECT_DIR/target/bench-tools"
TMP_OUTPUT="$TMP_DIR/criterion_output.txt"

mkdir -p "$TMP_DIR"

cd "$PROJECT_DIR"
echo "Running Criterion benchmarks... (output shown below)"
echo "=========================================="
cargo bench --bench js_benchmarks 2>&1 | tee "$TMP_OUTPUT"
python "$SCRIPT_DIR/drivers/update_latest_results.py" write-criterion "$REPORT_PATH" "$TMP_OUTPUT"

echo "Updated Criterion section in: $REPORT_PATH"
