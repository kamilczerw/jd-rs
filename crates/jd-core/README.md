# jd-core

Core library crate for the Rust port of the Go [`jd`](https://github.com/josephburnett/jd) JSON diff and patch tool.

## Usage

The library surface is under active development. For now it exposes a `version()` helper to enable smoke tests and doctests while the full data model is implemented in upcoming milestones.

Add the crate to your workspace using a path dependency:

```toml
[dependencies]
jd-core = { path = "../jd-core" }
```

## Examples

```rust
fn main() {
    println!("jd-core version {}", jd_core::version());
}
```

## Compatibility with Go jd

This crate targets feature parity with `jd` v2.2.2. The current milestone only scaffolds the workspace; functional parity work begins in Milestone 3 and beyond.
