# jd-fuzz

Fuzzing harness crate for the Rust port of the Go [`jd`](https://github.com/josephburnett/jd) project. The harnesses double as lightweight helpers for property-based smoke tests.

## Usage

The crate exposes three entry points suitable for `cargo fuzz` targets or manual invocation:

- `fuzz_canonicalization` — feeds arbitrary bytes through the JSON/YAML readers.
- `fuzz_diff` — generates random nodes and computes diffs/patches round-trips.
- `fuzz_patch` — applies both generated and arbitrary diffs to random documents.

When wiring a fuzz target, call the desired helper with the raw byte slice provided by `cargo fuzz`:

```rust
fn fuzz_target(data: &[u8]) {
    jd_fuzz::fuzz_diff(data);
}
```

## Compatibility with Go jd

The harnesses reuse the production `jd-core` types, ensuring every discovered crash or divergence maps directly to behavior present in the Go implementation. As additional diff modes and renderers land, new helpers will be added to maintain parity coverage.
