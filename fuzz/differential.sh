#!/usr/bin/env bash
set -euo pipefail

DURATION=${1:-60}
DIVERGENCES=0
RUNS=0
PANICS=0
LOG_FILE="fuzz/log.txt"
START=$(date +%s)

command -v node >/dev/null 2>&1 || { echo "node not found"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "cargo not found"; exit 1; }
[ -f "target/release/marked-rs" ] || cargo build --release

node -e "require('marked')" 2>/dev/null || {
    echo "Installing marked..."
    npm install marked
}

echo "Running differential fuzz for ${DURATION}s..." | tee "$LOG_FILE"
echo "Started: $(date)" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

while true; do
    NOW=$(date +%s)
    ELAPSED=$((NOW - START))
    [ $ELAPSED -ge $DURATION ] && break

    node fuzz/fuzz_gen.js > "fuzz/input.txt"
    INPUT=$(cat "fuzz/input.txt")

    RUST_OUT=$(./target/release/marked-rs < "fuzz/input.txt" 2>/dev/null || echo "PANIC")
    NODE_OUT=$(node --input-type=module -e "
import { marked } from 'marked';
import fs from 'fs';
let input = fs.readFileSync('fuzz/input.txt', 'utf8');
process.stdout.write(marked.parse(input));
" 2>/dev/null)

    RUNS=$((RUNS + 1))

    if [ "$RUST_OUT" = "PANIC" ]; then
        PANICS=$((PANICS + 1))
        echo "PANIC on run $RUNS" | tee -a "$LOG_FILE"
        printf "Input: %s\n" "$INPUT" >> "$LOG_FILE"
    elif [ "$RUST_OUT" != "$NODE_OUT" ]; then
        DIVERGENCES=$((DIVERGENCES + 1))
        {
            echo "=== DIVERGENCE #$DIVERGENCES (run $RUNS) ==="
            echo "INPUT: $INPUT"
            echo "RUST:  $RUST_OUT"
            echo "NODE:  $NODE_OUT"
            echo ""
        } | tee -a "$LOG_FILE"
    fi

    [ $((RUNS % 100)) -eq 0 ] && \
        echo "Progress: $RUNS runs, $DIVERGENCES divergences, ${ELAPSED}s elapsed"
done

{
    echo ""
    echo "=== FINAL RESULTS ==="
    echo "Duration:    ${DURATION}s"
    echo "Total runs:  $RUNS"
    echo "Panics:      $PANICS"
    echo "Divergences: $DIVERGENCES"
    echo "Finished:    $(date)"
    if [ $DIVERGENCES -eq 0 ] && [ $PANICS -eq 0 ]; then
        echo "STATUS: PASS — Zero divergences, zero panics"
    else
        echo "STATUS: FAIL"
    fi
} | tee -a "$LOG_FILE"
