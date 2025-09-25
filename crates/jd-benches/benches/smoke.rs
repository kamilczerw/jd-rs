use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use jd_benches::available_corpora;
use jd_core::{DiffOptions, RenderConfig};

fn bench_diff(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff");
    let options = DiffOptions::default();
    for corpus in available_corpora() {
        let dataset = corpus.load().expect("failed to load dataset");
        group.throughput(Throughput::Bytes(corpus.fixture_bytes() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(corpus.name()),
            &dataset,
            |b, dataset| {
                b.iter(|| {
                    let diff = dataset.diff(&options);
                    black_box(diff);
                });
            },
        );
    }
    group.finish();
}

fn bench_patch_apply(c: &mut Criterion) {
    let mut group = c.benchmark_group("patch-apply");
    let options = DiffOptions::default();
    for corpus in available_corpora() {
        let dataset = corpus.load().expect("failed to load dataset");
        let diff = dataset.diff(&options);
        group.throughput(Throughput::Bytes(corpus.fixture_bytes() as u64));
        group.bench_function(corpus.name(), {
            let dataset = dataset.clone();
            let diff = diff.clone();
            move |b| {
                b.iter(|| {
                    let result = dataset.before().apply_patch(&diff).expect("patch success");
                    black_box(result);
                });
            }
        });
    }
    group.finish();
}

fn bench_render(c: &mut Criterion) {
    let options = DiffOptions::default();
    let config = RenderConfig::default();

    {
        let mut native = c.benchmark_group("render-native");
        for corpus in available_corpora() {
            let dataset = corpus.load().expect("failed to load dataset");
            let diff = dataset.diff(&options);
            native.throughput(Throughput::Elements(diff.len() as u64));
            native.bench_function(corpus.name(), {
                let diff = diff.clone();
                move |b| {
                    b.iter(|| {
                        let rendered = diff.render(&config);
                        black_box(rendered);
                    });
                }
            });
        }
        native.finish();
    }

    {
        let mut json_patch = c.benchmark_group("render-json-patch");
        for corpus in available_corpora() {
            let dataset = corpus.load().expect("failed to load dataset");
            let diff = dataset.diff(&options);
            json_patch.throughput(Throughput::Elements(diff.len() as u64));
            json_patch.bench_function(corpus.name(), {
                let diff = diff.clone();
                move |b| {
                    b.iter(|| {
                        let rendered = diff.render_patch().expect("json patch");
                        black_box(rendered);
                    });
                }
            });
        }
        json_patch.finish();
    }
}

criterion_group!(benches, bench_diff, bench_patch_apply, bench_render);
criterion_main!(benches);
