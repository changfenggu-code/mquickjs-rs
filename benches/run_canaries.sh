#!/bin/bash
# Helper for the canonical Phase 1 benchmark canaries.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
MANIFEST="$SCRIPT_DIR/manifests/canary_benchmarks.txt"

usage() {
    cat <<EOF
Usage:
  ./benches/run_canaries.sh
      Run Criterion for the canonical canaries only.

  ./benches/run_canaries.sh --list
      Show the canonical canary manifest without running benchmarks.

  ./benches/run_canaries.sh --compare [compare.sh args...]
      Delegate to the canonical Rust-vs-C comparison flow in benches/compare.sh.
      Example: ./benches/run_canaries.sh --compare
               BENCH_RUNS=1 ./benches/run_canaries.sh --compare
               BENCH_EXEC_ITERS=100 ./benches/run_canaries.sh --compare --execution-only

  ./benches/run_canaries.sh --criterion
      Explicitly run the Criterion-only canary rerun flow.
EOF
}

require_manifest() {
    if [ ! -f "$MANIFEST" ]; then
        echo "Error: missing canary manifest: $MANIFEST"
        exit 1
    fi
}

load_manifest() {
    require_manifest
    CANARY_SCRIPTS=()
    CANARY_CRITERION_NAMES=()

    while IFS='|' read -r script_name criterion_name; do
        if [ -z "${script_name}${criterion_name}" ]; then
            continue
        fi
        case "$script_name" in
            \#*) continue ;;
        esac
        if [ -z "$script_name" ] || [ -z "$criterion_name" ]; then
            echo "Error: invalid manifest entry in $MANIFEST"
            exit 1
        fi
        if [ ! -f "$SCRIPT_DIR/workloads/${script_name}.js" ]; then
            echo "Error: missing benchmark script for manifest entry: $SCRIPT_DIR/workloads/${script_name}.js"
            exit 1
        fi
        CANARY_SCRIPTS+=("$script_name")
        CANARY_CRITERION_NAMES+=("$criterion_name")
    done < "$MANIFEST"

    if [ ${#CANARY_SCRIPTS[@]} -eq 0 ]; then
        echo "Error: no canaries were loaded from $MANIFEST"
        exit 1
    fi
}

list_canaries() {
    load_manifest
    for idx in "${!CANARY_SCRIPTS[@]}"; do
        printf "%s|%s\n" "${CANARY_SCRIPTS[$idx]}" "${CANARY_CRITERION_NAMES[$idx]}"
    done
}

run_criterion() {
    load_manifest
    cd "$PROJECT_DIR"
    for criterion_name in "${CANARY_CRITERION_NAMES[@]}"; do
        echo "==> cargo bench --bench js_benchmarks -- \"$criterion_name\""
        cargo bench --bench js_benchmarks -- "$criterion_name"
    done
}

MODE="criterion"

if [ $# -gt 0 ]; then
    case "$1" in
        --list)
            MODE="list"
            shift
            ;;
        --compare)
            MODE="compare"
            shift
            ;;
        --criterion)
            MODE="criterion"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Error: unknown argument: $1"
            usage
            exit 1
            ;;
    esac
fi

case "$MODE" in
    list)
        list_canaries
        ;;
    compare)
        exec "$SCRIPT_DIR/compare.sh" "$@"
        ;;
    criterion)
        run_criterion
        ;;
esac
