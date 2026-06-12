# Benchmark Methodology

## Environment

- **CPU**: To be filled at benchmark run (`lscpu` on Linux)
- **RAM**: To be filled at benchmark run
- **OS**: Windows 11 / Linux (CI: ubuntu-latest)
- **Rust**: `rustc 1.75.0` or later stable
- **Node**: v20+ for comparison benchmarks

## Build Configuration

```bash
cargo build --release
```

Release profile settings:
- `opt-level = 3`
- `lto = "fat"`
- `codegen-units = 1`
- `panic = "abort"`
- `strip = true`

## Throughput (Criterion)

```bash
make bench
```

- Benchmark group: `markdown_parse`
- Inputs: `bench/input/small.md` (10KB), `medium.md` (100KB), `large.md` (1MB)
- Throughput measured in bytes/second
- Criterion default sample size (100) with auto warm-up

### p99 Latency

Extract from `bench/criterion_output.txt` or HTML reports under `target/criterion/`.
Use the "time" estimate upper confidence bound at 99th percentile.

## Startup Time (hyperfine)

```bash
make bench-startup
```

- 5 warmup runs, 100 measured runs
- Command: `echo "# Hello" | ./target/release/marked-rs`

## Memory (RSS)

```bash
make bench-memory
```

Uses `/usr/bin/time -v` on Linux to report "Maximum resident set size".

## Conditions

- Run on AC power when possible
- Minimize background processes
- Close browser and IDE during measurement runs
