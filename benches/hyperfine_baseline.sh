#!/usr/bin/env bash
# hyperfine_baseline.sh — end-to-end process spawn benchmark for zacrs
# Usage: bash benches/hyperfine_baseline.sh
set -euo pipefail

BIN="target/release/zsh-autocomplete-rs"
FIXTURE_DIR="/tmp/zacrs-bench-fixtures"
RESULT_DIR="benches/results"

mkdir -p "$FIXTURE_DIR" "$RESULT_DIR"

# --- Generate TSV fixtures ---
generate_fixture() {
    local count=$1
    local file="$FIXTURE_DIR/${count}.tsv"
    if [[ -f "$file" ]]; then
        return
    fi
    local commands=(
        git git-add git-branch git-checkout git-clone git-commit git-diff
        git-fetch git-log git-merge git-pull git-push git-rebase git-reset
        cargo cargo-build cargo-check cargo-clippy cargo-test cargo-run
        ls cat grep find sed awk sort uniq head tail wc cut
        docker docker-compose npm npx node python python3 pip
        make cmake gcc curl wget ssh scp rsync chmod mkdir rm mv cp
    )
    local kinds=("command" "command" "command" "builtin" "function" "alias")
    : > "$file"
    for ((i = 0; i < count; i++)); do
        local base="${commands[$((i % ${#commands[@]}))]}"
        local kind="${kinds[$((i % ${#kinds[@]}))]}"
        if ((i >= ${#commands[@]})); then
            local suffix=$((i / ${#commands[@]}))
            echo -e "${base}-${suffix}\t${base} variant ${suffix}\t${kind}" >> "$file"
        else
            echo -e "${base}\t${base} command\t${kind}" >> "$file"
        fi
    done
    echo "Generated $file ($count candidates)"
}

generate_fixture 50
generate_fixture 200
generate_fixture 1000

# --- Build release binary ---
echo "Building release binary..."
CARGO_TARGET_DIR=target cargo build --release 2>&1 | tail -1

if [[ ! -x "$BIN" ]]; then
    echo "Error: $BIN not found"
    exit 1
fi

echo ""
echo "=== Baseline Benchmark (no daemon) ==="
echo ""

# --- Check hyperfine ---
if ! command -v hyperfine &>/dev/null; then
    echo "hyperfine not found. Install: cargo install hyperfine"
    exit 1
fi

# --- Benchmark: empty input (minimum overhead) ---
# render exits 1 with no candidates — that's expected
echo "--- Empty input (bare startup cost) ---"
hyperfine \
    --warmup 20 \
    --min-runs 100 \
    --ignore-failure \
    -N \
    "echo '' | $BIN render --prefix '' --cursor-row 5 --cursor-col 0" \
    --export-json "$RESULT_DIR/baseline_empty.json" \
    2>&1

echo ""

# --- Benchmark: 50 candidates ---
# render fails to open /dev/tty in non-interactive context — that's OK,
# we're measuring startup + parse + App::new (the parts daemon eliminates)
echo "--- 50 candidates ---"
hyperfine \
    --warmup 10 \
    --min-runs 50 \
    --ignore-failure \
    -N \
    "cat $FIXTURE_DIR/50.tsv | $BIN render --prefix gi --cursor-row 5 --cursor-col 2" \
    --export-json "$RESULT_DIR/baseline_50.json" \
    2>&1

echo ""

# --- Benchmark: 200 candidates ---
echo "--- 200 candidates ---"
hyperfine \
    --warmup 10 \
    --min-runs 50 \
    --ignore-failure \
    -N \
    "cat $FIXTURE_DIR/200.tsv | $BIN render --prefix gi --cursor-row 5 --cursor-col 2" \
    --export-json "$RESULT_DIR/baseline_200.json" \
    2>&1

echo ""

# --- Benchmark: 1000 candidates ---
echo "--- 1000 candidates ---"
hyperfine \
    --warmup 10 \
    --min-runs 50 \
    --ignore-failure \
    -N \
    "cat $FIXTURE_DIR/1000.tsv | $BIN render --prefix gi --cursor-row 5 --cursor-col 2" \
    --export-json "$RESULT_DIR/baseline_1000.json" \
    2>&1

echo ""
echo "=== Results saved to $RESULT_DIR/ ==="
