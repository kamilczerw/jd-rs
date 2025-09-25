# jd-cli

Command-line interface crate for the Rust port of the Go [`jd`](https://github.com/josephburnett/jd) JSON diff and patch tool.

## Usage

During the workspace scaffolding milestone the CLI binary only supports `--help` and `--version`. Future milestones will wire up the full flag surface and diff/patch functionality.

Run the binary with Cargo:

```bash
cargo run -p jd-cli -- --version
```

## Examples

```bash
$ cargo run -p jd-cli -- --help
Diff and patch JSON and YAML documents.
```

## Compatibility with Go jd

The CLI aims for byte-for-byte parity with `jd` v2.2.2. The current milestone provides the minimal scaffolding required to bootstrap tests and workflows; functional parity arrives in later milestones.
