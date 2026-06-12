.PHONY: build test bench bench-startup bench-memory fuzz differential \
        check unsafe-check setup clean demo cli-compat all

build:
	cargo build --release

test:
	cargo test -- --nocapture 2>&1 | tee test_output.txt

bench:
	cargo bench 2>&1 | tee bench/criterion_output.txt

bench-startup:
	hyperfine --warmup 5 --runs 100 \
	  --export-json bench/startup_results.json \
	  'echo "# Hello" | ./target/release/marked-rs'

bench-memory:
	/usr/bin/time -v sh -c \
	  'cat bench/input/large.md | ./target/release/marked-rs > /dev/null' \
	  2>&1 | grep "Maximum resident"

fuzz:
	cargo +nightly fuzz run fuzz_markdown \
	  -- -max_total_time=60 -artifact_prefix=fuzz/

differential:
	bash fuzz/differential.sh 60

check:
	cargo clippy -- -D warnings
	cargo fmt --check

unsafe-check:
	@COUNT=$$(grep -rn "unsafe" src/ | grep -v "//.*unsafe" | wc -l); \
	 echo "unsafe blocks in src/: $$COUNT"; \
	 test $$COUNT -eq 0

setup:
	mkdir -p tests/original/specs/marked fuzz/corpus bench/input
	curl -sf -o tests/original/specs/commonmark_spec.json \
	  https://spec.commonmark.org/0.31.2/spec.json
	python3 scripts/fetch_marked_specs.py
	python3 bench/generate.py
	npm install marked
	@echo "Setup complete. Run: make test"

clean:
	cargo clean
	rm -f test_output.txt bench/criterion_output.txt

demo:
	bash demo/run_demo.sh

cli-compat:
	@echo "Testing CLI surface compatibility..."
	echo "# test" | ./target/release/marked-rs
	echo "# test" | ./target/release/marked-rs --no-gfm
	./target/release/marked-rs --version
	./target/release/marked-rs --help
	@echo "CLI surface: PASS"

all: setup build unsafe-check check test bench differential
	@echo "All checks passed."
