#!/bin/bash
# Benchmark comparison script for MQuickJS-RS vs original MQuickJS
#
# Usage:
#   ./benches/compare.sh              # Auto-detect C version
#   ./benches/compare.sh /path/to/mqjs  # Use specific C binary
#
# Requirements:
#   - Rust toolchain (cargo)
#   - Optional: vendor/mquickjs submodule for C comparison

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BENCH_SCRIPTS="$SCRIPT_DIR/scripts"

# Detect Python (cross-platform)
if command -v python &> /dev/null; then
    PYTHON=python
elif command -v python3 &> /dev/null; then
    PYTHON=python3
else
    echo "Error: Python not found"
    exit 1
fi

echo "=== MQuickJS Benchmark Comparison ==="
echo ""
echo "Using: $PYTHON ($($PYTHON --version 2>&1))"
echo ""

# Check for C implementation
ORIGINAL_MQJS=""
if [ -n "$1" ]; then
    # User provided path
    ORIGINAL_MQJS="$1"
elif [ -x "$PROJECT_DIR/vendor/mquickjs/mqjs" ]; then
    # Try submodule
    ORIGINAL_MQJS="$PROJECT_DIR/vendor/mquickjs/mqjs"
fi

# Build Rust version
echo "[1/3] Building mquickjs-rs (release)..."
cd "$PROJECT_DIR"
cargo build --release --quiet
RUST_MQJS="$PROJECT_DIR/target/release/mqjs"

if [ ! -f "$RUST_MQJS" ]; then
    echo "Error: Failed to build mquickjs-rs"
    exit 1
fi

echo "      Built: $RUST_MQJS"
echo ""

# Check C version
if [ -z "$ORIGINAL_MQJS" ] || [ ! -x "$ORIGINAL_MQJS" ]; then
    echo "[2/3] C implementation not found. Running Rust-only benchmarks."
    echo "      To enable C comparison, run:"
    echo "        git submodule update --init"
    echo "      Or provide path: ./benches/compare.sh /path/to/mqjs"
    echo ""
    HAS_C=false
else
    echo "[2/3] Found C implementation: $ORIGINAL_MQJS"
    HAS_C=true
    echo ""
fi

# Function to run a benchmark
run_bench() {
    local script="$1"
    local binary="$2"
    local runs=5
    local total=0

    for i in $(seq 1 $runs); do
        local start=$($PYTHON -c 'import time; print(time.time())')
        "$binary" "$script" > /dev/null 2>&1
        local end=$($PYTHON -c 'import time; print(time.time())')
        local elapsed=$($PYTHON -c "print($end - $start)")
        total=$($PYTHON -c "print($total + $elapsed)")
    done

    $PYTHON -c "print(f'{$total / $runs:.4f}')"
}

# Run benchmarks
echo "[3/3] Running benchmarks (5 runs each, average)..."
echo ""
if [ "$HAS_C" = true ]; then
    echo "Benchmark               Rust (s)    C (s)      Ratio    Notes"
    echo "-----------------------------------------------------------------"
else
    echo "Benchmark               Rust (s)"
    echo "-----------------------------------------------------"
fi

for script in "$BENCH_SCRIPTS"/*.js; do
    name=$(basename "$script" .js)

    # Rust version
    rust_time=$(run_bench "$script" "$RUST_MQJS")

    if [ "$HAS_C" = true ]; then
        # Original version
        orig_time=$(run_bench "$script" "$ORIGINAL_MQJS")
        ratio=$($PYTHON -c "print(f'{$rust_time / $orig_time:.2f}x' if $orig_time > 0 else 'N/A')")

        # Determine notes
        if [ "$orig_time" != "0.0000" ]; then
            ratio_val=$($PYTHON -c "print($rust_time / $orig_time)")
            if $PYTHON -c "exit(0 if $ratio_val < 0.9 else 1)"; then
                notes="Rust faster"
            elif $PYTHON -c "exit(0 if $ratio_val > 1.1 else 1)"; then
                notes="C faster"
            else
                notes="~Equal"
            fi
        else
            notes="N/A"
        fi

        printf "  %-18s %10s %10s %10s    %s\n" "$name" "$rust_time" "$orig_time" "$ratio" "$notes"
    else
        printf "  %-18s %10s\n" "$name" "$rust_time"
    fi
done

echo ""
echo "Done!"
echo ""
echo "For detailed Rust benchmarks (Criterion), run:"
echo "  cargo bench"
