#!/usr/bin/env bash
#
# Compile-time benchmark for postbag vs other serde codecs.
#
# Measures how long it takes to compile 20 complex nested types
# with each codec in release mode. Each binary is touched and
# rebuilt to measure only the downstream crate compilation time
# (the codec library itself is already compiled).
#
# Usage:
#   ./bench.sh                  # run all benchmarks
#   ./bench.sh postbag_full     # run a single benchmark
#   ./bench.sh --debug          # measure debug builds instead

set -euo pipefail
cd "$(dirname "$0")"

PROFILE="release"
PROFILE_FLAG="--release"
TARGETS=(serde_only postbag_full postbag_slim bincode bincode2 serde_json)
BASE_RUSTFLAGS="${RUSTFLAGS:-}"

# Postbag targets that benefit from --cfg postbag_fast_compile.
FAST_COMPILE_TARGETS=(postbag_full postbag_slim)

for arg in "$@"; do
    case "$arg" in
        --debug)
            PROFILE="debug"
            PROFILE_FLAG=""
            ;;
        *)
            TARGETS=("$arg")
            ;;
    esac
done

bench_target() {
    local target="$1"
    local suffix="$2"
    local extra_rustflags="$3"

    export RUSTFLAGS="$BASE_RUSTFLAGS $extra_rustflags"

    # Ensure dependencies are pre-built.
    cargo build $PROFILE_FLAG --bin "$target" 2>/dev/null

    # Touch the binary source to force recompilation of only that crate.
    touch "src/${target}.rs"

    # Time the rebuild.
    start=$(date +%s%N)
    cargo build $PROFILE_FLAG --bin "$target" 2>/dev/null
    end=$(date +%s%N)
    elapsed=$(echo "scale=2; ($end - $start) / 1000000000" | bc)

    # Binary size.
    bin="target/${PROFILE}/${target}"
    if [[ -f "$bin" ]]; then
        size_kb=$(( $(stat -c%s "$bin") / 1024 ))
    else
        size_kb="-"
    fi

    local display_name="${target}${suffix}"
    printf "%-28s %8s %8s\n" "$display_name" "${elapsed}" "${size_kb}"
}

is_fast_compile_target() {
    local target="$1"
    for fc in "${FAST_COMPILE_TARGETS[@]}"; do
        [[ "$fc" == "$target" ]] && return 0
    done
    return 1
}

echo "Pre-building dependencies ($PROFILE)..."
export RUSTFLAGS="$BASE_RUSTFLAGS"
for t in "${TARGETS[@]}"; do
    cargo build $PROFILE_FLAG --bin "$t" 2>/dev/null
done

# Run all binaries to verify correctness before benchmarking.
echo "Verifying correctness..."
for target in "${TARGETS[@]}"; do
    "target/${PROFILE}/${target}"
    if is_fast_compile_target "$target"; then
        export RUSTFLAGS="$BASE_RUSTFLAGS --cfg postbag_fast_compile"
        cargo build $PROFILE_FLAG --bin "$target" 2>/dev/null
        "target/${PROFILE}/${target}"
        export RUSTFLAGS="$BASE_RUSTFLAGS"
    fi
done
echo ""

printf "%-28s %8s %8s\n" "Target" "Time(s)" "Size(KB)"
printf "%-28s %8s %8s\n" "------" "-------" "--------"

for target in "${TARGETS[@]}"; do
    bench_target "$target" "" ""
    if is_fast_compile_target "$target"; then
        bench_target "$target" " (fast-compile)" "--cfg postbag_fast_compile"
    fi
done
