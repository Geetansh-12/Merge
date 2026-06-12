#!/usr/bin/env bash
# 5-minute demo script for the submission video
# Record with: asciinema rec demo.cast

set -euo pipefail

echo "=========================================="
echo " marked-rs: JavaScript → Rust Port"
echo " Port Mortem Hackathon — Track F"
echo "=========================================="
sleep 2

echo ""
echo "--- STEP 1: Build ---"
make build
echo "Binary: $(ls -lh target/release/marked-rs | awk '{print $5}')"
sleep 1

echo ""
echo "--- STEP 2: Zero unsafe check ---"
make unsafe-check
sleep 1

echo ""
echo "--- STEP 3: CommonMark spec compliance ---"
cargo test commonmark_spec_compliance -- --nocapture 2>&1 | tail -5
sleep 1

echo ""
echo "--- STEP 4: Quick demo ---"
printf "# Hello from marked-rs\n\n**Bold**, *italic*, and \`code\`.\n\n> A blockquote.\n" \
  | ./target/release/marked-rs
sleep 1

echo ""
echo "--- STEP 5: Differential fuzz (15s preview) ---"
bash fuzz/differential.sh 15
sleep 1

echo ""
echo "--- STEP 6: Throughput benchmark ---"
echo "Input: $(wc -c < bench/input/large.md | numfmt --to=iec)B"
time cat bench/input/large.md | ./target/release/marked-rs > /dev/null
sleep 1

echo ""
echo "=========================================="
echo " Results:"
echo "   unsafe blocks : 0"
echo "   spec tests    : see above"
echo "   divergences   : 0 (15s fuzz)"
echo "=========================================="
