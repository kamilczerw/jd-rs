use criterion::{criterion_group, criterion_main, Criterion};

fn bench_version(c: &mut Criterion) {
    c.bench_function("jd_core_version", |b| b.iter(jd_core::version));
}

criterion_group!(benches, bench_version);
criterion_main!(benches);
