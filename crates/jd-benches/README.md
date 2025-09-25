# jd-benches

Benchmark harness crate for the Rust port of the Go [`jd`](https://github.com/josephburnett/jd) tool.

## Usage

No benchmarks are defined yet; the crate is part of the workspace scaffolding so that Criterion-based suites can be added in the dedicated performance milestone.

## Examples

```rust
assert!(!jd_benches::is_ready());
```

## Compatibility with Go jd

Future benchmarks will measure parity scenarios against the Go implementation to verify performance targets. During the scaffolding milestone this crate intentionally contains only placeholders.
