# jd-benches

Benchmark harness crate for the Rust port of the Go [`jd`](https://github.com/josephburnett/jd) tool. The crate packages representative corpora and helper utilities consumed by the Criterion suites under `benches/` and by parity comparison scripts.

## Usage

Run the Criterion suite locally:

```console
$ cargo bench -p jd-benches
```

The crate ships three canonical corpora (`kubernetes-deployment`, `github-issue`, `large-array`) sourced from the Go repository. Each corpus exposes helper methods to load canonicalized `Node`s, compute diffs, and render outputs.

## Examples

```rust
use jd_benches::available_corpora;
use jd_core::{DiffOptions, RenderConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let corpus = available_corpora()
        .iter()
        .find(|c| c.name() == "github-issue")
        .expect("registered corpus");
    let dataset = corpus.load()?;
    let diff = dataset.diff(&DiffOptions::default());
    assert!(!diff.is_empty());

    let rendered = dataset.render_native(&diff, &RenderConfig::default());
    assert!(rendered.contains("@ "));
    Ok(())
}
```

## Compatibility with Go jd

Use `scripts/bench_vs_go.sh` to compare the Rust CLI (`cargo build --release -p jd-cli`) with the Go 2.2.2 binary on the same corpora. The script records wall time and peak RSS for both implementations, enabling parity tracking across releases.
