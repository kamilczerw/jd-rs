# Status — Milestone 1 (Preparation & Recon Sync)

## Summary
- Re-read the committed implementation plan to reconfirm milestone ordering and deliverables.
- Audited upstream `jd` Go CLI (v2.2.2 tag) to restate the authoritative surface area we must mirror.
- Noted canonicalization behavior around empty inputs and YAML parsing for later parity tests.

## Findings & References
1. **CLI flags and modes** – Confirmed the v2 CLI exposes `-color`, `-f`, `-git-diff-driver`, `-mset`, `-o`, `-p`, `-port`, `-precision`, `-set`, `-setkeys`, `-t`, `-version`, and `-yaml`, plus a no-op `-v2` shim. These options govern diff/patch/translate/web UI paths and must be mirrored exactly in the Rust port, including mutual exclusion of `-p` and `-t`. [(v2.2.2/jd/main.go#L20-L115)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/jd/main.go#L20-L115)
2. **Exit codes and diff detection** – Verified that diff mode exits with status 1 when the rendered diff is non-empty, otherwise 0, while patch/translate exit 0 on success; git diff driver path enforces seven git-provided arguments. [(v2.2.2/jd/main.go#L185-L244)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/jd/main.go#L185-L244)
3. **Input canonicalization baseline** – Empty or whitespace-only payloads produce a `voidNode`, JSON/YAML parsing uses `encoding/json` and `gopkg.in/yaml.v2`, and YAML maps with non-string keys are rejected via `NewJsonNode`. This informs our Rust canonicalization and error parity plan. [(v2.2.2/v2/node_read.go#L10-L48)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/node_read.go#L10-L48), [(v2.2.2/v2/node.go#L10-L72)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/node.go#L10-L72)

## Status — Milestone 2 (Workspace Scaffolding Prep)

### Summary
- Reviewed the upstream CLI control flow again to capture precise usage text, exit statuses, and argument handling that the Rust scaffolding must mimic out of the gate.
- Confirmed metadata flag validation (`-precision` incompatibilities, `-setkeys` parsing) and diff/patch render semantics to ensure early scaffolding includes placeholders for these options.
- Re-checked input handling pathways (file vs stdin, YAML toggles) so initial CLI smoke tests target the same IO permutations as the Go binary.

### Findings & References
1. **Usage banner and exit code** – `printUsageAndExit` renders a fixed banner (notably including trailing blank lines and examples) before exiting with status 2; scaffolding help snapshots must account for this verbatim layout. [(v2.2.2/v2/jd/main.go#L126-L177)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/jd/main.go#L126-L177)
2. **Metadata validation rules** – `parseMetadata` forbids combining `-precision` with `-set`/`-mset`, normalizes `-setkeys`, and appends render options in a deterministic order; Rust option parsing needs equivalent guards. [(v2.2.2/v2/jd/main.go#L66-L122)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/jd/main.go#L66-L122)
3. **Diff/Patch IO semantics** – Diff mode exits 1 when a diff exists, 0 otherwise; patch/translate always exit 0 on success while routing YAML vs JSON parsing through dedicated helpers. File vs stdin routing matches mode selection logic we should emulate in early CLI scaffolding tests. [(v2.2.2/v2/jd/main.go#L34-L125)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/jd/main.go#L34-L125)
4. **Canonicalization baseline** – Empty or whitespace-only inputs map to `voidNode{}` across JSON and YAML readers, reinforcing the need for canonicalization utilities that treat whitespace-only files as void. [(v2.2.2/v2/node_read.go#L10-L45)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/node_read.go#L10-L45)

### Next Steps
- Implement Milestone 2 workspace scaffolding with tooling configuration, ensuring initial CLI stubs and smoke tests reflect the documented usage/help structure.
- Capture any scaffolding-related deviations as ADRs before proceeding to code-level changes.
