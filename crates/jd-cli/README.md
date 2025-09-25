# jd-cli

Command-line interface for the Rust port of the Go [`jd`](https://github.com/josephburnett/jd) JSON diff and patch tool. The binary wires the `jd-core` crate into a parity-focused CLI experience.

## Usage

Run the binary with Cargo while the project iterates toward packaged releases:

```console
$ cargo run -p jd-cli -- --help
```

Key flags implemented so far:

- `--version` – print `jd version <semver>` and exit.
- `--format {jd,patch,merge}` / `-f` – select native jd, JSON Patch, or JSON Merge Patch rendering.
- `--color` – enable ANSI color sequences for native format output.
- Positional arguments (`FILE1 [FILE2]`) mirroring Go `jd` diff semantics, with `-` representing STDIN.

Patch/translate/git-diff-driver/web modes are acknowledged but will emit informative errors until their milestones land.

## Examples

```console
$ cat <<'EOF' > /tmp/before.json
{"name":"old"}
EOF
$ cat <<'EOF' > /tmp/after.json
{"name":"new"}
EOF
$ cargo run -p jd-cli -- /tmp/before.json /tmp/after.json
@ ["name"]
- "old"
+ "new"
$ cargo run -p jd-cli -- --format patch /tmp/before.json /tmp/after.json
[{"op":"test","path":"/name","value":"old"},{"op":"remove","path":"/name","value":"old"},{"op":"add","path":"/name","value":"new"}]
```

## Compatibility with Go jd

The CLI mirrors Go `jd` v2.2.2 help text, exit codes, diff detection logic, and rendering byte-for-byte for the supported flags. Future milestones will extend parity coverage to patch/translate modes, git diff driver integration, and the web UI shim.
