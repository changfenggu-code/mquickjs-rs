#!/bin/bash
# Benchmark comparison script for MQuickJS-RS vs original MQuickJS
#
# Usage:
#   ./benches/compare.sh              # Auto-detect C version
#   ./benches/compare.sh /path/to/mqjs  # Use specific C binary
#
# Requirements:
#   - Rust toolchain (cargo)
#   - Optional: contrib/mquickjs submodule for C comparison
#   - Optional: GCC (for building C version on Windows)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BENCH_SCRIPTS="$SCRIPT_DIR/scripts"

# Detect platform
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) EXE=".exe"; IS_WIN=true ;;
    *)                     EXE="";     IS_WIN=false ;;
esac

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
echo "Platform: $(uname -s), Python: $($PYTHON --version 2>&1)"
echo ""

# --- Locate or build C implementation ---
ORIGINAL_MQJS=""

if [ -n "$1" ]; then
    # User provided path
    ORIGINAL_MQJS="$1"
elif [ -x "$PROJECT_DIR/contrib/mquickjs/bin/mqjs${EXE}" ]; then
    ORIGINAL_MQJS="$PROJECT_DIR/contrib/mquickjs/bin/mqjs${EXE}"
elif [ -x "$PROJECT_DIR/contrib/mquickjs/mqjs${EXE}" ]; then
    ORIGINAL_MQJS="$PROJECT_DIR/contrib/mquickjs/mqjs${EXE}"
elif [ -f "$PROJECT_DIR/contrib/mquickjs/mquickjs.c" ]; then
    # Submodule exists but binary not built — try to build it
    echo "[*] C source found but no binary. Attempting to build..."
    VENDOR_DIR="$PROJECT_DIR/contrib/mquickjs"

    if command -v make &> /dev/null && [ "$IS_WIN" = false ]; then
        # Unix: use Makefile
        make -C "$VENDOR_DIR" -j"$(nproc 2>/dev/null || echo 4)" mqjs 2>&1 | tail -3
        if [ -x "$VENDOR_DIR/mqjs" ]; then
            ORIGINAL_MQJS="$VENDOR_DIR/mqjs"
        fi
    elif command -v gcc &> /dev/null; then
        # Windows or no make: compile with gcc directly
        mkdir -p "$VENDOR_DIR/bin"
        CFLAGS="-Os -Wall -D_GNU_SOURCE -fno-math-errno -fno-trapping-math"
        C_SRCS="mqjs mquickjs dtoa libm cutils readline readline_tty"

        echo "    Compiling C sources..."
        for src in $C_SRCS; do
            gcc $CFLAGS -c "$VENDOR_DIR/${src}.c" -o "$VENDOR_DIR/bin/${src}.o" 2>/dev/null
        done

        echo "    Linking mqjs${EXE}..."
        LDLIBS="-lm"
        if [ "$IS_WIN" = true ]; then
            LDLIBS="$LDLIBS -lpthread"  # MinGW64 needs -lpthread for nanosleep
        fi
        if gcc -g -o "$VENDOR_DIR/bin/mqjs${EXE}" \
            "$VENDOR_DIR"/bin/mqjs.o \
            "$VENDOR_DIR"/bin/mquickjs.o \
            "$VENDOR_DIR"/bin/dtoa.o \
            "$VENDOR_DIR"/bin/libm.o \
            "$VENDOR_DIR"/bin/cutils.o \
            "$VENDOR_DIR"/bin/readline.o \
            "$VENDOR_DIR"/bin/readline_tty.o \
            $LDLIBS 2>/dev/null; then
            ORIGINAL_MQJS="$VENDOR_DIR/bin/mqjs${EXE}"
            echo "    Built: $ORIGINAL_MQJS"
        else
            echo "    Warning: C build failed. Running Rust-only benchmarks."
        fi
    else
        echo "    Warning: No GCC found. Cannot build C version."
    fi
fi

# --- Build Rust version ---
echo "[1/3] Building mquickjs-rs (release)..."
cd "$PROJECT_DIR"
cargo build --release --quiet
RUST_MQJS="$PROJECT_DIR/target/release/mqjs${EXE}"

if [ ! -f "$RUST_MQJS" ]; then
    echo "Error: Failed to build mquickjs-rs"
    exit 1
fi

echo "      Rust: $RUST_MQJS"

# Check C version
if [ -n "$ORIGINAL_MQJS" ] && [ -f "$ORIGINAL_MQJS" ]; then
    # Verify it's actually a JS engine (not mqjs_stdlib)
    if "$ORIGINAL_MQJS" -e "print(1)" > /dev/null 2>&1; then
        echo "      C:    $ORIGINAL_MQJS"
        HAS_C=true
    else
        echo "      Warning: $ORIGINAL_MQJS is not a valid JS engine. Skipping C comparison."
        HAS_C=false
    fi
else
    echo "[2/3] C implementation not found. Running Rust-only benchmarks."
    echo "      To enable C comparison:"
    echo "        git submodule update --init"
    echo "      Or provide path: ./benches/compare.sh /path/to/mqjs"
    HAS_C=false
fi
echo ""

# --- Benchmark runner ---
RUNS=15  # More runs for stability

run_bench() {
    local script="$1"
    local binary="$2"
    local total=0

    for i in $(seq 1 $RUNS); do
        local start=$($PYTHON -c 'import time; print(time.time())')
        "$binary" "$script" > /dev/null 2>&1
        local end=$($PYTHON -c 'import time; print(time.time())')
        local elapsed=$($PYTHON -c "print($end - $start)")
        total=$($PYTHON -c "print($total + $elapsed)")
    done

    $PYTHON -c "print(f'{$total / $RUNS:.4f}')"
}

# --- Run benchmarks ---
echo "[3/3] Running benchmarks ($RUNS runs each, average)..."
echo ""
if [ "$HAS_C" = true ]; then
    printf "  %-18s %10s %10s %10s    %s\n" "Benchmark" "Rust (s)" "C (s)" "Ratio" "Notes"
    echo "  -----------------------------------------------------------------"
else
    printf "  %-18s %10s\n" "Benchmark" "Rust (s)"
    echo "  -----------------------------------"
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
