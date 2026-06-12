use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use std::fs;

fn bench_parse(c: &mut Criterion) {
    let inputs = [
        ("small_10kb", "bench/input/small.md"),
        ("medium_100kb", "bench/input/medium.md"),
        ("large_1mb", "bench/input/large.md"),
    ];

    let mut group = c.benchmark_group("markdown_parse");

    for (name, path) in &inputs {
        match fs::read_to_string(path) {
            Ok(content) => {
                group.throughput(Throughput::Bytes(content.len() as u64));
                group.bench_with_input(
                    BenchmarkId::new("marked_rs", name),
                    &content,
                    |b, content| b.iter(|| marked_rs::parse(black_box(content))),
                );
            }
            Err(_) => eprintln!("Skipping {}: run `python3 bench/generate.py` first", path),
        }
    }
    group.finish();
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
