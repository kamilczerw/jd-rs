# jd-rs

Rust workspace for the port of the Go [`jd`](https://github.com/josephburnett/jd) JSON diff & patch tool. The implementation follows the milestone plan captured in [`docs/`](docs/) and aims for byte-for-byte parity with `jd` v2.2.2 across rendering formats, patch semantics, and CLI behavior.

## Installation

The workspace is split into multiple crates. The CLI can be installed from the repository once Rust 1.74+ is available:

```console
$ cargo install --path crates/jd-cli
```

Developers typically work with the workspace directly:

```console
$ cargo fmt
$ cargo clippy --all-targets --all-features -- -D warnings
$ cargo test --all --all-features
```

## Quickstart

The CLI currently supports diffing two JSON or YAML documents and rendering the result using the native jd format, JSON Patch, or JSON Merge Patch. The example below compares two JSON snippets and writes a merge patch to stdout:

```console
$ cat <<'EOF' > /tmp/before.json
{"name":"jd","version":1}
EOF
$ cat <<'EOF' > /tmp/after.json
{"name":"jd","version":2}
EOF
$ cargo run -p jd-cli -- --format merge /tmp/before.json /tmp/after.json
{"name":"jd","version":2}
```

The core library can be embedded directly when programmatic access to the diff engine is required:

```rust
use jd_core::{DiffOptions, Node};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = Node::from_json_str("{\"count\":1}")?;
    let target = Node::from_json_str("{\"count\":2}")?;

    let diff = base.diff(&target, &DiffOptions::default());
    assert!(!diff.is_empty());

    let patched = base.apply_patch(&diff)?;
    assert_eq!(patched, target);
    Ok(())
}
```

## Compatibility with Go jd

Feature work marches through the milestones documented in [`docs/implementation_plan.md`](docs/implementation_plan.md). Every user-visible deviation from the Go binary must be justified with an ADR and backed by a failing parity test. The current state provides:

- Canonical JSON/YAML parsing mirroring Go's whitespace, numeric, and key-handling rules.
- Structural diffing for list-mode arrays, objects, and scalars.
- Patch application and reverse-diff support with strict/merge semantics.
- Renderers for native jd format, JSON Patch, and JSON Merge Patch.
- CLI parity for `jd --help`, `--version`, diff rendering selection (`-f`), and color toggling (`--color`).

See [`docs/status.md`](docs/status.md) for milestone-by-milestone recon notes.

## Workspace layout

```
crates/
├─ jd-core      # Core library (data model, diff, patch, renderers)
├─ jd-cli       # Command-line interface binary
├─ jd-fuzz      # Fuzzing harnesses (cargo-fuzz)
└─ jd-benches   # Criterion benchmarks and Go parity runners
```

Additional scripts for regenerating golden fixtures and parity tests live under [`scripts/`](scripts/).

## Contributing

Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) for workflow expectations, including mandatory ADRs for notable decisions, documentation requirements, and testing strategy. A summarized change history is maintained in [`CHANGELOG.md`](CHANGELOG.md).
