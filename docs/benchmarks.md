# Benchmark Baselines

This document captures the initial performance baselines for the Rust `jd` port. The benchmarks exercise three corpora shared with the Go implementation and live under `crates/jd-benches/fixtures/`:

- `kubernetes-deployment` – representative `apps/v1` Deployment manifest changes.
- `github-issue` – webhook payload evolution for an issue close event.
- `large-array` – synthetic array workload stressing hashing and LCS traversal.

## Criterion results

Run the full suite with:

```shell
cargo bench -p jd-benches
```

The table below records the median runtime for each benchmark group (Criterion reports minimum/median/maximum); values are in microseconds unless otherwise noted.

| Corpus | Diff (`jd_core::Node::diff`) | Patch Apply (`Node::apply_patch`) | Render Native (`Diff::render`) | Render JSON Patch (`Diff::render_patch`) |
| --- | --- | --- | --- | --- |
| kubernetes-deployment | 44.79 µs | 382.10 µs | 27.40 µs | 36.63 µs |
| github-issue | 13.42 µs | 22.93 µs | 33.76 µs | 12.79 µs |
| large-array | 1.1519 ms | 46.01 ms | 135.26 µs | 330.10 µs |

Criterion emitted a warning for the `large-array` diff benchmark indicating the default sampling window was tight; the measurement still completed with 100 samples but required ~5.7s to gather.【d59fe6†L4-L8】【5f0e9d†L1-L5】

Source output for the timing summaries is linked below for traceability.【68c13b†L1-L6】【d59fe6†L1-L8】【5f0e9d†L1-L5】【c14715†L1-L5】【0548ae†L1-L5】【37ed80†L1-L5】【692ff3†L1-L3】【eae215†L1-L5】【a2d9b2†L1-L4】【621317†L1-L6】【ce1dd7†L1-L3】【79d8c9†L1-L4】

## Rust vs Go CLI parity harness

`scripts/bench_vs_go.sh` builds both CLIs, executes the diff mode on each corpus, and records wall time plus peak RSS (via `/usr/bin/time` when available, or a Python `resource` fallback). Example run on this environment:

```shell
./scripts/bench_vs_go.sh
```

_Output excerpt:_

```
warning: /usr/bin/time not found; falling back to Python resource metrics
Binary       Corpus                   Seconds    MaxRSS(KB)   Exit
rust         github                   0.006201   12288        0
go           github                   0.020546   12312        0

rust         kubernetes               0.007215   12324        0
go           kubernetes               0.020379   12080        0

rust         large-array              0.015443   12140        0
go           large-array              0.048685   12256        0
```

The Python fallback (triggered here because `/usr/bin/time` is unavailable) reports elapsed seconds using `time.perf_counter` and RSS via `resource.getrusage`. Exit code `0` reflects that both CLIs normalise the inputs identically with no structural differences detected for these fixtures.【9a0bb5†L1-L4】【16aad0†L1-L6】【e0c508†L1-L2】

