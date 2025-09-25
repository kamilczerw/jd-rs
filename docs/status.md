# Status — Milestone 1 (Preparation & Recon Sync)

## Summary
- Re-read the committed implementation plan to reconfirm milestone ordering and deliverables.
- Audited upstream `jd` Go CLI (v2.2.2 tag) to restate the authoritative surface area we must mirror.
- Noted canonicalization behavior around empty inputs and YAML parsing for later parity tests.

## Findings & References
1. **CLI flags and modes** – Confirmed the v2 CLI exposes `-color`, `-f`, `-git-diff-driver`, `-mset`, `-o`, `-p`, `-port`, `-precision`, `-set`, `-setkeys`, `-t`, `-version`, and `-yaml`, plus a no-op `-v2` shim. These options govern diff/patch/translate/web UI paths and must be mirrored exactly in the Rust port, including mutual exclusion of `-p` and `-t`. [(v2.2.2/jd/main.go#L20-L115)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/jd/main.go#L20-L115)
2. **Exit codes and diff detection** – Verified that diff mode exits with status 1 when the rendered diff is non-empty, otherwise 0, while patch/translate exit 0 on success; git diff driver path enforces seven git-provided arguments. [(v2.2.2/jd/main.go#L185-L244)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/jd/main.go#L185-L244)
3. **Input canonicalization baseline** – Empty or whitespace-only payloads produce a `voidNode`, JSON/YAML parsing uses `encoding/json` and `gopkg.in/yaml.v2`, and YAML maps with non-string keys are rejected via `NewJsonNode`. This informs our Rust canonicalization and error parity plan. [(v2.2.2/v2/node_read.go#L10-L48)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/node_read.go#L10-L48), [(v2.2.2/v2/node.go#L10-L72)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/node.go#L10-L72)

## Next Steps
- Proceed to Milestone 2: scaffold the Cargo workspace exactly per implementation plan, ensuring tooling configs (`rust-toolchain`, fmt, clippy, deny) are added and smoke tests for `--help`/`--version` are outlined.
- Capture any deviations or open questions discovered during scaffolding as ADRs before implementation.
