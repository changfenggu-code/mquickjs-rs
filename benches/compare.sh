#!/bin/bash
# Benchmark comparison script for MQuickJS-RS vs the checked-in C reference tree.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BENCH_SCRIPTS="$SCRIPT_DIR/workloads"
CANARY_MANIFEST="$SCRIPT_DIR/manifests/canary_benchmarks.txt"
TIER_MANIFEST="$SCRIPT_DIR/manifests/benchmark_tiers.txt"
EXEC_MEM_MANIFEST="$SCRIPT_DIR/manifests/execution_mem_sizes.txt"
C_EXEC_HELPER_SOURCE="$SCRIPT_DIR/drivers/bench_exec_helper.c"
BENCH_TOOL_DIR="$PROJECT_DIR/target/bench-tools"
MQJS_BUILD_DIR="$BENCH_TOOL_DIR/mqjs-build"
CONTRIB_CODEGEN_DIR="$MQJS_BUILD_DIR/codegen"
CONTRIB_MQJS_BUILD_DIR="$MQJS_BUILD_DIR/mqjs"
CONTRIB_EXEC_HELPER_DIR="$MQJS_BUILD_DIR"
RUNS="${BENCH_RUNS:-15}"
EXEC_ITERS="${BENCH_EXEC_ITERS:-50}"
REPORT_PATH="${BENCH_REPORT_PATH:-$SCRIPT_DIR/LATEST_RESULTS.md}"
REPORT_SECTION_PATH="$BENCH_TOOL_DIR/latest-results-section.md"
REPORT_UPDATER="$SCRIPT_DIR/drivers/update_latest_results.py"

case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) EXE=".exe"; IS_WIN=true ;;
    *)                     EXE="";     IS_WIN=false ;;
esac

if command -v python >/dev/null 2>&1; then
    PYTHON=python
elif command -v python3 >/dev/null 2>&1; then
    PYTHON=python3
else
    echo "Error: Python not found"
    exit 1
fi

usage() {
    cat <<EOF
Usage:
  ./benches/compare.sh
      Run the canonical Phase 1 canary comparison from $CANARY_MANIFEST
      using the checked-in contrib/mquickjs reference tree.

  ./benches/compare.sh --execution-only
      Run compile-once / execute-many Rust-vs-C comparison for the selected
      benchmarks. This aligns with the Criterion-style runtime focus.

  ./benches/compare.sh --all
      Run every script in benches/workloads with the checked-in contrib/mquickjs
      reference tree. This is broader than the canonical Phase 1 canary flow.

  ./benches/compare.sh --tier 3
      Run one benchmark tier from $TIER_MANIFEST. This is useful for focused
      diagnostic passes without falling back to the entire script directory.

  ./benches/compare.sh --list-tiers
      Print the current tier manifest.

  ./benches/compare.sh --rust-only
      Run the selected benchmark set in explicit Rust-only mode.

  ./benches/compare.sh --all --rust-only
      Run every local benchmark script in non-canonical Rust-only mode.

  ./benches/compare.sh --c-binary /path/to/mqjs
      Use a custom C CLI binary in non-canonical end-to-end mode.

  BENCH_EXEC_ITERS=100 ./benches/compare.sh --execution-only
      Override the in-process iteration count used by execution-only helpers.
EOF
}

require_file() {
    local path="$1"
    local message="$2"
    if [ ! -f "$path" ]; then
        echo "Error: $message"
        exit 1
    fi
}

report_begin() {
    local mode_label="$1"
    local metric_label="$2"
    local selection_label="$3"
    local rust_runner="$4"
    local c_runner="$5"
    local generated_at

    generated_at="$($PYTHON -c 'from datetime import datetime; print(datetime.now().astimezone().strftime("%Y-%m-%d %H:%M:%S %z"))')"
    mkdir -p "$BENCH_TOOL_DIR"

    cat > "$REPORT_SECTION_PATH" <<EOF
- Generated at: \`$generated_at\`
- Platform: \`$(uname -s)\`
- Python: \`$($PYTHON --version 2>&1)\`
- Mode: \`$mode_label\`
- Metric: \`$metric_label\`
- Selection: \`$selection_label\`
- Rust runner: \`$rust_runner\`
- C runner: \`$c_runner\`

| Benchmark | Rust (s) | C (s) | Ratio | Notes |
|-----------|----------|-------|-------|-------|
EOF
}

report_add_row() {
    local name="$1"
    local rust_time="$2"
    local c_time="$3"
    local ratio="$4"
    local notes="$5"
    printf '| %s | %s | %s | %s | %s |\n' "$name" "$rust_time" "$c_time" "$ratio" "$notes" >> "$REPORT_SECTION_PATH"
}

report_commit() {
    local section_key="$1"
    cat >> "$REPORT_SECTION_PATH" <<'EOF'

## Notes

- This section is updated by `benches/compare.sh`.
EOF
    python "$REPORT_UPDATER" write-section "$REPORT_PATH" "$section_key" "$REPORT_SECTION_PATH"
}

validate_positive_integer() {
    local name="$1"
    local value="$2"
    if ! printf '%s' "$value" | grep -Eq '^[1-9][0-9]*$'; then
        echo "Error: $name must be a positive integer, got: $value"
        exit 1
    fi
}

benchmark_mem_size() {
    local script_name="$1"
    local line
    line="$(grep -E "^${script_name}\\|" "$EXEC_MEM_MANIFEST" || true)"
    if [ -z "$line" ]; then
        echo "Error: missing execution memory config for workload: $script_name"
        exit 1
    fi
    printf '%s\n' "${line#*|}"
}

validate_engine_binary() {
    local candidate="$1"
    [ -x "$candidate" ] || return 1
    "$candidate" -e "print(1)" >/dev/null 2>&1
}

ensure_contrib_codegen() {
    local vendor_dir="$PROJECT_DIR/contrib/mquickjs"
    local host_exe="$CONTRIB_CODEGEN_DIR/mqjs_stdlib.host${EXE}"
    local host_cflags="-Wall -g -D_GNU_SOURCE -fno-math-errno -fno-trapping-math -O2"

    mkdir -p "$CONTRIB_CODEGEN_DIR"

    if [ -f "$CONTRIB_CODEGEN_DIR/mqjs_stdlib.h" ] && [ -f "$CONTRIB_CODEGEN_DIR/mquickjs_atom.h" ]; then
        return 0
    fi

    if command -v make >/dev/null 2>&1 && [ "$IS_WIN" = false ]; then
        make -C "$vendor_dir" mqjs_stdlib.h mquickjs_atom.h >/dev/null
        cp "$vendor_dir/mqjs_stdlib.h" "$CONTRIB_CODEGEN_DIR/mqjs_stdlib.h"
        cp "$vendor_dir/mquickjs_atom.h" "$CONTRIB_CODEGEN_DIR/mquickjs_atom.h"
        return 0
    fi

    if ! command -v gcc >/dev/null 2>&1; then
        return 1
    fi

    gcc $host_cflags -c "$vendor_dir/mqjs_stdlib.c" -o "$CONTRIB_CODEGEN_DIR/mqjs_stdlib.host.o"
    gcc $host_cflags -c "$vendor_dir/mquickjs_build.c" -o "$CONTRIB_CODEGEN_DIR/mquickjs_build.host.o"
    gcc -g -o "$host_exe" \
        "$CONTRIB_CODEGEN_DIR/mqjs_stdlib.host.o" \
        "$CONTRIB_CODEGEN_DIR/mquickjs_build.host.o"

    "$host_exe" -a > "$CONTRIB_CODEGEN_DIR/mquickjs_atom.h"
    "$host_exe" > "$CONTRIB_CODEGEN_DIR/mqjs_stdlib.h"
}

build_contrib_mquickjs() {
    local vendor_dir="$PROJECT_DIR/contrib/mquickjs"
    local build_dir="$CONTRIB_MQJS_BUILD_DIR"

    ensure_contrib_codegen || return 1

    if command -v make >/dev/null 2>&1 && [ "$IS_WIN" = false ]; then
        make -C "$vendor_dir" -j"$(nproc 2>/dev/null || echo 4)" mqjs >/dev/null
        if validate_engine_binary "$vendor_dir/bin/mqjs${EXE}"; then
            printf '%s\n' "$vendor_dir/bin/mqjs${EXE}"
            return 0
        fi
        if validate_engine_binary "$vendor_dir/mqjs${EXE}"; then
            printf '%s\n' "$vendor_dir/mqjs${EXE}"
            return 0
        fi
        return 1
    fi

    if ! command -v gcc >/dev/null 2>&1; then
        return 1
    fi

    mkdir -p "$build_dir"
    local cflags="-Os -Wall -D_GNU_SOURCE -fno-math-errno -fno-trapping-math -I$CONTRIB_CODEGEN_DIR -I$vendor_dir"
    local sources="mqjs mquickjs dtoa libm cutils readline readline_tty"

    for src in $sources; do
        gcc $cflags -c "$vendor_dir/${src}.c" -o "$build_dir/${src}.o"
    done

    local ldlibs="-lm"
    if [ "$IS_WIN" = true ]; then
        ldlibs="$ldlibs -lpthread"
    fi

    gcc -g -o "$build_dir/mqjs${EXE}" \
        "$build_dir/mqjs.o" \
        "$build_dir/mquickjs.o" \
        "$build_dir/dtoa.o" \
        "$build_dir/libm.o" \
        "$build_dir/cutils.o" \
        "$build_dir/readline.o" \
        "$build_dir/readline_tty.o" \
        $ldlibs >/dev/null

    printf '%s\n' "$build_dir/mqjs${EXE}"
}

resolve_contrib_mquickjs() {
    local vendor_dir="$PROJECT_DIR/contrib/mquickjs"

    if validate_engine_binary "$vendor_dir/bin/mqjs${EXE}"; then
        printf '%s\n' "$vendor_dir/bin/mqjs${EXE}"
        return 0
    fi

    if validate_engine_binary "$vendor_dir/mqjs${EXE}"; then
        printf '%s\n' "$vendor_dir/mqjs${EXE}"
        return 0
    fi

    if [ -f "$vendor_dir/mquickjs.c" ]; then
        local built
        if built="$(build_contrib_mquickjs)" && validate_engine_binary "$built"; then
            printf '%s\n' "$built"
            return 0
        fi
    fi

    return 1
}

build_contrib_exec_helper() {
    local vendor_dir="$PROJECT_DIR/contrib/mquickjs"
    local helper_dir="$CONTRIB_EXEC_HELPER_DIR"
    local helper_path="$helper_dir/bench_exec_helper${EXE}"
    local cflags="-Os -Wall -D_GNU_SOURCE -fno-math-errno -fno-trapping-math"
    local ldlibs="-lm"

    ensure_contrib_codegen || return 1
    if ! command -v gcc >/dev/null 2>&1; then
        return 1
    fi

    mkdir -p "$helper_dir"
    if [ "$IS_WIN" = true ]; then
        ldlibs="$ldlibs -lpthread"
    fi

    gcc $cflags -I"$vendor_dir" -o "$helper_path" \
        -I"$CONTRIB_CODEGEN_DIR" \
        "$C_EXEC_HELPER_SOURCE" \
        "$vendor_dir/mquickjs.c" \
        "$vendor_dir/dtoa.c" \
        "$vendor_dir/libm.c" \
        "$vendor_dir/cutils.c" \
        $ldlibs >/dev/null

    printf '%s\n' "$helper_path"
}

run_end_to_end_bench() {
    local script="$1"
    local binary="$2"
    local total=0

    for i in $(seq 1 "$RUNS"); do
        local start end elapsed
        start=$($PYTHON -c 'import time; print(time.time())')
        "$binary" "$script" >/dev/null 2>&1
        end=$($PYTHON -c 'import time; print(time.time())')
        elapsed=$($PYTHON -c "print($end - $start)")
        total=$($PYTHON -c "print($total + $elapsed)")
    done

    $PYTHON -c "print(f'{$total / $RUNS:.4f}')"
}

run_exec_bench() {
    local helper="$1"
    local script="$2"
    local mem_size="$3"
    "$helper" "$script" "$EXEC_ITERS" "$mem_size"
}

declare -a BENCHMARK_SCRIPTS=()
declare -a BENCHMARK_NAMES=()

USE_ALL=false
LIST_TIERS=false
ALLOW_RUST_ONLY=false
EXECUTION_ONLY=false
CUSTOM_C_BINARY=""
SELECTED_TIER=""

while [ $# -gt 0 ]; do
    case "$1" in
        --all)
            USE_ALL=true
            shift
            ;;
        --tier)
            if [ $# -lt 2 ]; then
                echo "Error: --tier requires a tier number"
                exit 1
            fi
            SELECTED_TIER="$2"
            shift 2
            ;;
        --list-tiers)
            LIST_TIERS=true
            shift
            ;;
        --rust-only)
            ALLOW_RUST_ONLY=true
            shift
            ;;
        --execution-only)
            EXECUTION_ONLY=true
            shift
            ;;
        --c-binary)
            if [ $# -lt 2 ]; then
                echo "Error: --c-binary requires a path"
                exit 1
            fi
            CUSTOM_C_BINARY="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            if [ -z "$CUSTOM_C_BINARY" ] && [ -e "$1" ]; then
                CUSTOM_C_BINARY="$1"
                shift
            else
                echo "Error: Unknown argument: $1"
                usage
                exit 1
            fi
            ;;
    esac
done

if [ "$USE_ALL" = true ] && [ -n "$SELECTED_TIER" ]; then
    echo "Error: --all and --tier cannot be combined"
    exit 1
fi

if [ "$EXECUTION_ONLY" = true ] && [ -n "$CUSTOM_C_BINARY" ]; then
    echo "Error: --c-binary is not supported with --execution-only"
    exit 1
fi

if [ "$LIST_TIERS" = true ] && { [ "$USE_ALL" = true ] || [ -n "$SELECTED_TIER" ] || [ "$ALLOW_RUST_ONLY" = true ] || [ -n "$CUSTOM_C_BINARY" ] || [ "$EXECUTION_ONLY" = true ]; }; then
    echo "Error: --list-tiers does not take benchmark execution options"
    exit 1
fi

if [ "$LIST_TIERS" = true ]; then
    require_file "$TIER_MANIFEST" "benchmark tier manifest not found at $TIER_MANIFEST"
    cat "$TIER_MANIFEST"
    exit 0
fi

NON_CANONICAL_MODE=false
if [ "$USE_ALL" = true ] || [ "$ALLOW_RUST_ONLY" = true ] || [ -n "$CUSTOM_C_BINARY" ] || [ -n "$SELECTED_TIER" ] || [ "$EXECUTION_ONLY" = true ]; then
    NON_CANONICAL_MODE=true
fi

require_file "$CANARY_MANIFEST" "canonical benchmark manifest not found at $CANARY_MANIFEST"
require_file "$TIER_MANIFEST" "benchmark tier manifest not found at $TIER_MANIFEST"
require_file "$EXEC_MEM_MANIFEST" "execution memory manifest not found at $EXEC_MEM_MANIFEST"

if [ -n "$SELECTED_TIER" ] && ! printf '%s' "$SELECTED_TIER" | grep -Eq '^[0-9]+$'; then
    echo "Error: tier must be numeric"
    exit 1
fi

validate_positive_integer "BENCH_RUNS" "$RUNS"
validate_positive_integer "BENCH_EXEC_ITERS" "$EXEC_ITERS"

while IFS='|' read -r script_name criterion_name; do
    if [ -z "${script_name}${criterion_name}" ]; then
        continue
    fi

    case "$script_name" in
        \#*) continue ;;
    esac

    if [ -z "$script_name" ] || [ -z "$criterion_name" ]; then
        echo "Error: invalid manifest entry in $CANARY_MANIFEST"
        exit 1
    fi

    require_file "$BENCH_SCRIPTS/$script_name.js" "manifest entry $script_name does not have a matching JS benchmark script"

    if [ "$USE_ALL" = false ] && [ -z "$SELECTED_TIER" ]; then
        BENCHMARK_SCRIPTS+=("$BENCH_SCRIPTS/$script_name.js")
        BENCHMARK_NAMES+=("$criterion_name")
    fi
done < "$CANARY_MANIFEST"

if [ -n "$SELECTED_TIER" ]; then
    while IFS='|' read -r tier script_name display_name; do
        if [ -z "${tier}${script_name}${display_name}" ]; then
            continue
        fi

        case "$tier" in
            \#*) continue ;;
        esac

        if [ -z "$tier" ] || [ -z "$script_name" ] || [ -z "$display_name" ]; then
            echo "Error: invalid tier manifest entry in $TIER_MANIFEST"
            exit 1
        fi

        if [ "$tier" != "$SELECTED_TIER" ]; then
            continue
        fi

        require_file "$BENCH_SCRIPTS/$script_name.js" "tier entry $script_name does not have a matching JS benchmark script"
        BENCHMARK_SCRIPTS+=("$BENCH_SCRIPTS/$script_name.js")
        BENCHMARK_NAMES+=("$display_name")
    done < "$TIER_MANIFEST"
fi

if [ "$USE_ALL" = true ]; then
    for script in "$BENCH_SCRIPTS"/*.js; do
        require_file "$script" "benchmark script missing: $script"
        BENCHMARK_SCRIPTS+=("$script")
        BENCHMARK_NAMES+=("$(basename "$script" .js)")
    done
fi

if [ ${#BENCHMARK_SCRIPTS[@]} -eq 0 ]; then
    echo "Error: no benchmark scripts selected"
    exit 1
fi

echo "=== MQuickJS Benchmark Comparison ==="
echo
echo "Platform: $(uname -s), Python: $($PYTHON --version 2>&1)"
if [ "$NON_CANONICAL_MODE" = true ]; then
    echo "Mode: non-canonical"
    if [ -n "$SELECTED_TIER" ]; then
        echo "Tier: $SELECTED_TIER (from $TIER_MANIFEST)"
    fi
else
    echo "Mode: canonical Phase 1 canary comparison from $CANARY_MANIFEST"
fi
if [ "$EXECUTION_ONLY" = true ]; then
    echo "Metric: compile-once / execute-many average (${EXEC_ITERS} in-process iterations)"
    METRIC_LABEL="compile-once / execute-many average (${EXEC_ITERS} in-process iterations)"
else
    echo "Metric: end-to-end process execution average (${RUNS} process runs)"
    METRIC_LABEL="end-to-end process execution average (${RUNS} process runs)"
fi
echo

echo "[1/3] Building mquickjs-rs (release)..."
cd "$PROJECT_DIR"
if [ "$EXECUTION_ONLY" = true ]; then
    cargo build --release --quiet --bin mqjs --bin bench_exec_helper
else
    cargo build --release --quiet --bin mqjs
fi

RUST_RUNNER=""
if [ "$EXECUTION_ONLY" = true ]; then
    RUST_RUNNER="$PROJECT_DIR/target/release/bench_exec_helper${EXE}"
else
    RUST_RUNNER="$PROJECT_DIR/target/release/mqjs${EXE}"
fi
require_file "$RUST_RUNNER" "failed to build Rust benchmark runner: $RUST_RUNNER"
echo "      Rust: $RUST_RUNNER"

HAS_C=false
C_RUNNER=""
if [ "$ALLOW_RUST_ONLY" = false ]; then
    if [ "$EXECUTION_ONLY" = true ]; then
        if [ -f "$PROJECT_DIR/contrib/mquickjs/mquickjs.c" ]; then
            if C_RUNNER="$(build_contrib_exec_helper)"; then
                HAS_C=true
            fi
        fi
        if [ "$HAS_C" = false ]; then
            echo "Error: execution-only Rust-vs-C comparison requires a buildable contrib/mquickjs helper."
            echo "Action:"
            echo "  1. Ensure contrib/mquickjs is present"
            echo "  2. Ensure gcc is available"
            echo "  3. Or rerun explicitly with --rust-only"
            exit 1
        fi
    else
        if [ -n "$CUSTOM_C_BINARY" ]; then
            C_RUNNER="$CUSTOM_C_BINARY"
            if ! validate_engine_binary "$C_RUNNER"; then
                echo "Error: custom C binary is not runnable: $C_RUNNER"
                exit 1
            fi
            HAS_C=true
        else
            if C_RUNNER="$(resolve_contrib_mquickjs)"; then
                HAS_C=true
            else
                echo "Error: canonical Rust-vs-C benchmark comparison requires the checked-in contrib/mquickjs reference tree."
                echo "Action:"
                echo "  1. Ensure contrib/mquickjs is present: git submodule update --init --recursive"
                echo "  2. Build it if needed: make -C contrib/mquickjs mqjs"
                echo "  3. Or rerun explicitly in non-canonical mode with --rust-only"
                exit 1
            fi
        fi
    fi
fi

echo "[2/3] Resolving C benchmark runner..."
if [ "$HAS_C" = true ]; then
    echo "      C:    $C_RUNNER"
else
    echo "      C:    skipped (--rust-only)"
fi
echo

if [ "$NON_CANONICAL_MODE" = true ]; then
    MODE_LABEL="non-canonical"
else
    MODE_LABEL="canonical canary"
fi

if [ "$USE_ALL" = true ]; then
    SELECTION_LABEL="all workloads"
elif [ -n "$SELECTED_TIER" ]; then
    SELECTION_LABEL="tier $SELECTED_TIER"
else
    SELECTION_LABEL="canary manifest"
fi

if [ "$HAS_C" = true ]; then
    REPORT_C_RUNNER="$C_RUNNER"
else
    REPORT_C_RUNNER="rust-only"
fi

if [ "$EXECUTION_ONLY" = true ]; then
    REPORT_SECTION_KEY="execution_only"
else
    REPORT_SECTION_KEY="end_to_end"
fi

report_begin "$MODE_LABEL" "$METRIC_LABEL" "$SELECTION_LABEL" "$RUST_RUNNER" "$REPORT_C_RUNNER"

echo "[3/3] Running benchmarks..."
echo
if [ "$HAS_C" = true ]; then
    printf "  %-28s %10s %10s %10s    %s\n" "Benchmark" "Rust (s)" "C (s)" "Ratio" "Notes"
    echo "  -------------------------------------------------------------------------------"
else
    printf "  %-28s %10s\n" "Benchmark" "Rust (s)"
    echo "  ---------------------------------------------"
fi

if [ "$EXECUTION_ONLY" = false ] && [ "$HAS_C" = true ] && [ "$USE_ALL" = false ] && [ -z "$SELECTED_TIER" ]; then
    rust_start_total=0
    for i in $(seq 1 "$RUNS"); do
        start=$($PYTHON -c 'import time; print(time.time())')
        "$RUST_RUNNER" -e "0" > /dev/null 2>&1 || true
        end=$($PYTHON -c 'import time; print(time.time())')
        elapsed=$($PYTHON -c "print($end - $start)")
        rust_start_total=$($PYTHON -c "print($rust_start_total + $elapsed)")
    done
    rust_start=$($PYTHON -c "print(f'{$rust_start_total / $RUNS:.4f}')")

    c_start_total=0
    for i in $(seq 1 "$RUNS"); do
        start=$($PYTHON -c 'import time; print(time.time())')
        "$C_RUNNER" -e "0" > /dev/null 2>&1 || true
        end=$($PYTHON -c 'import time; print(time.time())')
        elapsed=$($PYTHON -c "print($end - $start)")
        c_start_total=$($PYTHON -c "print($c_start_total + $elapsed)")
    done
    c_start=$($PYTHON -c "print(f'{$c_start_total / $RUNS:.4f}')")
    start_ratio=$($PYTHON -c "r=$rust_start; c=$c_start; print(f'{r/c:.2f}x' if c > 0 else 'N/A')")
    printf "  %-28s %10s %10s %10s    %s\n" "startup_baseline" "$rust_start" "$c_start" "$start_ratio" "process startup"
    report_add_row "startup_baseline" "$rust_start" "$c_start" "$start_ratio" "process startup"
fi

for idx in "${!BENCHMARK_SCRIPTS[@]}"; do
    script="${BENCHMARK_SCRIPTS[$idx]}"
    name="${BENCHMARK_NAMES[$idx]}"
    script_name="$(basename "$script" .js)"
    mem_size="$(benchmark_mem_size "$script_name")"

    if [ "$EXECUTION_ONLY" = true ]; then
        rust_time="$(run_exec_bench "$RUST_RUNNER" "$script" "$mem_size")"
    else
        rust_time="$(run_end_to_end_bench "$script" "$RUST_RUNNER")"
    fi

    if [ "$HAS_C" = true ]; then
        if [ "$EXECUTION_ONLY" = true ]; then
            c_time="$(run_exec_bench "$C_RUNNER" "$script" "$mem_size")"
        else
            c_time="$(run_end_to_end_bench "$script" "$C_RUNNER")"
        fi
        ratio=$($PYTHON -c "print(f'{$rust_time / $c_time:.2f}x' if $c_time > 0 else 'N/A')")

        if $PYTHON -c "import math; c=$c_time; raise SystemExit(0 if c > 0 else 1)"; then
            ratio_val=$($PYTHON -c "print($rust_time / $c_time)")
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

        printf "  %-28s %10s %10s %10s    %s\n" "$name" "$rust_time" "$c_time" "$ratio" "$notes"
        report_add_row "$name" "$rust_time" "$c_time" "$ratio" "$notes"
    else
        printf "  %-28s %10s\n" "$name" "$rust_time"
        report_add_row "$name" "$rust_time" "-" "-" "rust-only"
    fi
done

report_commit "$REPORT_SECTION_KEY"

echo
echo "Done!"
echo "Latest report: $REPORT_PATH"
echo
echo "For detailed Rust benchmarks (Criterion), run:"
echo "  cargo bench --bench js_benchmarks"
