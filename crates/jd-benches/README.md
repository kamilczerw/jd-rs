# jd-benches

Benchmark harness crate for the Rust port of the Go [`jd`](https://github.com/josephburnett/jd) tool.

## Usage

Run the Criterion suite with:

```shell
cargo bench -p jd-benches
```

Benchmarks cover three corpora (`kubernetes-deployment`, `github-issue`, `large-array`) that mirror real workloads captured in `crates/jd-benches/fixtures/`.

## Examples

```rust
use jd_benches::available_corpora;
use jd_core::DiffOptions;

let corpus = available_corpora().iter().find(|c| c.name() == "large-array").unwrap();
let dataset = corpus.load().unwrap();
let diff = dataset.diff(&DiffOptions::default());
assert!(diff.len() > 0);
```

## Compatibility with Go jd

Use `scripts/bench_vs_go.sh` to compare the Rust CLI (`cargo build --release -p jd-cli`) with the Go 2.2.2 binary on the same corpora. The script records wall time and peak RSS for both implementations, enabling parity tracking across releases.
