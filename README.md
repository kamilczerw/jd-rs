# jd-rs

Rust workspace for the port of the Go [`jd`](https://github.com/josephburnett/jd) JSON diff & patch tool. The project follows the milestone plan in [`docs/`](docs/) and targets byte-for-byte output parity with `jd` v2.2.2.

## Usage

The current milestone focuses on scaffolding. You can build the workspace and exercise the CLI stub:

```bash
cargo fmt
cargo test
cargo run -p jd-cli -- --help
```

## Examples

```bash
$ cargo run -p jd-cli -- --version
jd version 0.0.0
```

## Compatibility with Go jd

Parity work is tracked milestone-by-milestone. During the scaffolding phase only the CLI skeleton and crate placeholders exist; functional parity efforts begin with the data model milestone.

## Workspace layout

```
crates/
├─ jd-core      # Core library (data model, diff, patch — forthcoming)
├─ jd-cli       # Command-line interface
├─ jd-fuzz      # Fuzzing harnesses (to be implemented)
└─ jd-benches   # Criterion benchmarks (to be implemented)
```
