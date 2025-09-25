# jd-fuzz

Fuzzing harness crate for the Rust port of the Go [`jd`](https://github.com/josephburnett/jd) project.

## Usage

The crate currently only provides scaffolding so that the workspace builds cleanly. Dedicated `cargo-fuzz` targets will be added in Milestone 5 once the core data model and diff implementation are available.

## Examples

```rust
assert!(!jd_fuzz::is_ready());
```

## Compatibility with Go jd

Fuzz targets will exercise the same canonicalization, diff, and patch code paths as the Go implementation to ensure behavioral parity. No fuzzing functionality is available yet during the scaffolding milestone.
