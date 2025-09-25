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

## Status — Milestone 3 (Data Model & Canonicalization Recon)

### Summary
- Re-read the upstream node construction code to confirm how Go normalizes native maps, slices, and scalars into JsonNode implementations prior to diffing.
- Revalidated the JSON/YAML reader helpers so our Rust canonicalization layer mirrors whitespace handling, error propagation, and the void sentinel semantics.
- Cataloged scalar behaviors (numbers, strings, void) to ensure our tests cover precision tolerances and empty-input rendering before we start coding the Rust equivalents.

### Findings & References
1. **Node construction surface** – `NewJsonNode` recursively converts `map[string]interface{}` and `[]interface{}` values into concrete node types, while rejecting YAML-style `map[interface{}]interface{}` keys that are not strings. This guarantees canonical object keys and is the behavior we must replicate. [(v2.2.2/v2/node.go#L8-L86)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/node.go#L8-L86)
2. **Whitespace-to-void canonicalization** – The shared `unmarshal` helper trims input and returns `voidNode{}` when the payload is empty or whitespace-only before calling the JSON/YAML decoder. Errors bubble directly from the decoder. [(v2.2.2/v2/node_read.go#L8-L44)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/node_read.go#L8-L44)
3. **Numeric equality semantics** – Numeric nodes are stored as `float64` (`jsonNumber`) and equality tolerates differences up to the user-provided `-precision` option (default 0). We'll need parity tests around this float-based tolerance. [(v2.2.2/v2/number.go#L5-L44)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/number.go#L5-L44)
4. **Void rendering** – `voidNode` renders to empty JSON/YAML strings and carries a deterministic hash, which impacts canonical rendering tests and diff behavior when inputs are missing. [(v2.2.2/v2/void.go#L1-L46)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/void.go#L1-L46)

### Next Steps
- Draft Milestone 3 tests for node constructors, number precision, and whitespace canonicalization before implementing the Rust data model.
- Raise ADRs if Rust numeric representation or YAML decoding requires divergence from the Go float64 + gopkg.v2 approach.
