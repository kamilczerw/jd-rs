# jd-core

Core library for the Rust port of the Go [`jd`](https://github.com/josephburnett/jd) JSON diff and patch tool. The crate exposes the canonical data model (`Node`), diff representation (`Diff`/`DiffElement`), patch application helpers, and renderer APIs used by the CLI and integration tests.

## Usage

Add the crate to a workspace using a path dependency while it iterates toward its first crates.io release:

```toml
[dependencies]
jd-core = { path = "../jd-core" }
```

The entry point for most workflows is [`Node`], which can be parsed from JSON or YAML strings and then diffed/ patched:

```rust
use jd_core::{DiffOptions, Node};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = Node::from_json_str("[1,2,3]")?;
    let target = Node::from_json_str("[1,4,3]")?;

    let diff = base.diff(&target, &DiffOptions::default());
    assert_eq!(diff.len(), 1);

    let json_patch = diff.render_patch()?;
    assert!(json_patch.contains("\"op\":\"test\""));

    let patched = base.apply_patch(&diff)?;
    assert_eq!(patched, target);
    Ok(())
}
```

See the crate-level rustdoc for additional examples covering merge semantics, metadata propagation, and diff rendering.

## Compatibility with Go jd

The implementation targets Go `jd` v2.2.2 semantics:

- Canonicalization mirrors Go's whitespace, numeric, and YAML key handling.
- Diff output (native, JSON Patch, JSON Merge Patch) matches byte-for-byte on the curated parity corpus.
- Patch application enforces the same before/after context validation and strict vs merge strategies.

Any intentional divergence requires an ADR under [`ADRs/`](../../ADRs/).
