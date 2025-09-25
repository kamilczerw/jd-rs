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

## Status — Milestone 4 (Diff Engine Recon)

### Summary
- Re-read the Go `DiffElement` definition to restate how path segments and list context (`Before`/`After`) encode everything required for patch application.
- Traced the list-mode diff to confirm it hashes elements, runs an LCS to align common subsequences, and stitches nested container diffs in place while tracking positional context.
- Verified object diffs walk keys in lexical order, emitting removals/additions with merge metadata parity when one side lacks a key.
- Confirmed `diffRest`'s path cursor math and context recording (including the `+2` increment when appending) so our Rust implementation preserves Go's insertion points and `Before`/`After` behavior exactly.
- Checked `golcs`'s reconstruction loop to ensure ties prefer consuming from the left array, matching the deterministic ordering relied upon by the Go diff.

### Findings & References
1. **DiffElement structure** – Each hunk carries metadata, a fully qualified path, and optional `Before`/`After` context arrays used only for list diffs. This matches `diff.go`'s definition and establishes the Rust struct fields we must mirror. [(v2.2.2/v2/diff.go#L3-L44)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/diff.go#L3-L44)
2. **List diff algorithm** – The Go list diff hashes elements, computes an LCS via `golcs`, and then iterates with cursors to accumulate add/remove hunks, inserting nested diffs when encountering compatible containers and recording context for patch validation. [(v2.2.2/v2/list.go#L69-L249)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/list.go#L69-L249)
3. **Object diff ordering and metadata** – Objects sort both key sets, recurse when keys exist on both sides, and otherwise emit deletion/addition hunks (switching to merge metadata when requested). [(v2.2.2/v2/object.go#L123-L209)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/object.go#L123-L209)
4. **Path cursor & context semantics** – `diffRest` advances the cursor by two when accumulating pure appends and records `Before`/`After` context relative to the common subsequence index, falling back to `voidNode{}` when the RHS lacks trailing context. [(v2.2.2/v2/list.go#L109-L235)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/list.go#L109-L235)
5. **LCS tie-breaking** – The upstream `golcs` implementation decrements the left index first when the DP table entries are equal, giving deterministic matches that favor earlier elements; the Rust LCS helper must mimic this behavior. [(golcs/golcs.go#L102-L124)](https://github.com/yudai/golcs/blob/master/golcs.go#L102-L124)

### Next Steps
- Draft the Milestone 4 diff engine component spec capturing struct definitions, algorithm stages, and hashing dependencies before writing tests.
- Identify fixtures from the Go repository to snapshot as golden outputs for list-mode diff parity, then script their regeneration.

## Status — Milestone 5 (Patch Apply & Reverse Recon)

### Summary
- Inspected the Go patch application pipeline to understand how metadata selects between strict and merge semantics and how patch elements are applied sequentially.
- Traced list and object patchers to capture context validation, append handling, and merge-specific object creation requirements.
- Reviewed the upstream diff/patch end-to-end tests to restate invariants we must satisfy when wiring the Rust patch engine and reverse operations.

### Findings & References
1. **Patch dispatch strategy** – `patchAll` iterates diff elements, switching between strict and merge strategies based on metadata before delegating to node-specific `patch` implementations, while enforcing single-value replacements for non-set modes. [(v2.2.2/patch_common.go#L7-L65)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/patch_common.go#L7-L65)
2. **List patch semantics** – `jsonList.patch` handles whole-list replacements, recursive descent, append-at-`-1`, before/after context verification, and strict removal validation with detailed error strings. [(v2.2.2/v2/list.go#L313-L415)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/list.go#L313-L415)
3. **Object patch semantics** – `jsonObject.patch` checks merge metadata, ensures strict replacements verify prior values, auto-creates nested objects for merge strategy, and deletes entries when child patches return void. [(v2.2.2/v2/object.go#L216-L271)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/object.go#L216-L271)

### Additional Findings (Milestone 5 Follow-up)
- **Strategy inheritance & merge path constraints** – Confirmed `patchAll` clones an empty path for each element and `patch` enforces merge-only traversal beyond leaves, rejecting non-string keys during merge descent. [(v2.2.2/v2/patch_common.go#L7-L45)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/patch_common.go#L7-L45)
- **List context enforcement order** – Verified list patching checks before-context indices relative to insertion point prior to executing removals, mirroring Go's `invalid patch. before context …` errors and ensuring `-1` append forbids removals. [(v2.2.2/v2/list.go#L323-L409)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/list.go#L323-L409)
- **Merge object materialization** – Noted object patch creates intermediate objects (or void leaves) under merge strategy when keys are absent, then deletes map entries whenever recursive patches return `void`. [(v2.2.2/v2/object.go#L244-L271)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/object.go#L244-L271)
4. **End-to-end invariants** – `TestDiffAndPatch` exercises diff→patch round-trips (including JSON Patch rendering) establishing parity expectations for success and failure cases we must replicate. [(v2.2.2/v2/e2e_test.go#L7-L159)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/e2e_test.go#L7-L159)

### Next Steps
- Design Milestone 5 tests covering node-level patching, diff round-trips, and reverse operations before implementing the Rust patch engine.
- Document any deviations in ADRs if Rust requires structural changes (e.g., iterator ownership vs borrowing) while maintaining parity with Go error messages.

## Status — Milestone 6 (Patch Renderers & Reverse Recon)

### Summary
- Re-read the Go renderer implementation to catalog native jd formatting, JSON Patch sequencing, and merge serialization prerequisites before porting the logic.
- Confirmed JSON Pointer rules and context-validation errors surfaced by the Go code so tests can assert byte-for-byte parity.
- Noted that the plan's `-test` flag mention maps to internal reverse-diff checks because the upstream CLI exposes no such flag; captured this in ADR 0001.
- Validated reverse diff behavior matches Go by iterating elements in reverse order, swapping additions/removals, and inheriting metadata explicitly so property-based round-trips succeed.
- Re-checked merge rendering in Go and confirmed `RenderMerge` funnels through `json.Marshal`, which collapses integral floats to integer tokens, so Rust canonicalization must emit `5` instead of `5.0` for merge fixtures.

### Findings & References
1. **Native renderer layout & color diffing** – `DiffElement.Render` emits metadata headers, list context, and colorized single-string diffs using an LCS of runes; our port must replicate the exact control flow and ANSI codes.【0e54f4†L18-L156】
2. **JSON Patch conversion constraints** – `Diff.RenderPatch` enforces single-line before/after context, validates array indices, injects `test/remove/add` operations in order, and rejects empty hunks; these semantics define our error messages.【0e54f4†L166-L268】
3. **Merge patch rendering** – `Diff.RenderMerge` requires every element (including inherited metadata) to be marked merge, coerces void additions to null, and patches against a void node to reuse canonical writers.【d20d2f†L270-L289】
4. **Pointer encoding & patch parsing** – `ReadPatchString` and `writePointer` restrict numeric-looking object keys and assemble pointer strings with RFC 6901 escaping, establishing our pointer helpers and validation paths.【8daf59†L223-L320】【84cad1†L10-L41】
5. **CLI surface confirmation** – Upstream CLI flags exclude any `-test` option, so parity work must stay within the documented surface (`-p`, `-f`, `-t`, etc.).【074fa5†L23-L199】
6. **Diff exit code semantics** – Confirmed the Go CLI determines non-empty diffs by checking native output for empty string, JSON Patch for "[]", and merge patch for "{}" before returning exit status 1. This logic informs the Rust CLI parity implementation.【d0b38f†L1-L61】
7. **Merge JSON canonicalization** – `Diff.RenderMerge` patches a void node and then calls `mergePatch.Json()`, which serializes via `encoding/json` and inherently strips redundant decimal suffixes from integers; Rust must mirror this behavior in its `Node::to_json_value` helper.【d003bd†L260-L280】

### Next Steps
- Expand renderer coverage with larger fixtures and cross-implementation parity snapshots, especially for set/multiset semantics slated for later milestones.
- Begin wiring CLI render mode flags (`-n`, `-p`, `-m`, etc.) to the new renderer APIs while preserving merge/set metadata behavior for future work.

## Status — Milestone 7 (CLI Parity & UX Recon)

### Summary
- Re-verified the Go CLI flag definitions, usage banner, and mutually exclusive mode handling so the Rust CLI can mirror them exactly.
- Confirmed argument arity rules for diff, patch, translate, and git diff driver modes, including stdout/stderr/exit-code semantics for each path.
- Documented that colorized output is only enabled when `-color` is passed; there is no upstream `-nocolor` flag or `NO_COLOR` environment override.

### Findings & References
1. **Flag surface & version handling** – The CLI exposes the `-color`, `-f`, `-git-diff-driver`, `-mset`, `-o`, `-p`, `-port`, `-precision`, `-set`, `-setkeys`, `-t`, `-version`, `-yaml`, and deprecated `-v2` flags; `-version` prints `jd version 2.2.2` and exits. Mode selection rejects simultaneous `-p` and `-t`. [(v2.2.2/v2/jd/main.go#L18-L111)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/jd/main.go#L18-L111)
2. **Usage banner & exit codes** – `printUsageAndExit` emits the documented multi-line banner (including blank lines and examples) before exiting with status 2. [(v2.2.2/v2/jd/main.go#L157-L199)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/jd/main.go#L157-L199)
3. **Argument arity & IO routing** – Diff/patch modes accept one or two positional arguments (second input defaults to STDIN), translate accepts zero or one, and git diff driver requires seven arguments; diff exits 1 when changes are detected, otherwise 0, while patch/translate always exit 0 on success. [(v2.2.2/v2/jd/main.go#L72-L379)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/jd/main.go#L72-L379)
4. **Color behavior** – The renderer only adds `jd.COLOR` when `-color` is supplied; there is no automatic TTY detection or `NO_COLOR` handling in upstream. [(v2.2.2/v2/jd/main.go#L229-L283)](https://github.com/josephburnett/jd/blob/v2.2.2/v2/jd/main.go#L229-L283)

### Next Steps
- Mirror these flag definitions and mutually exclusive combinations in the Clap-based parser.
- Add argument arity validation tests and snapshots that assert exit codes and help/usage text match upstream.
- Reflect the absence of auto color toggles when reconciling the implementation plan for Milestone 7.
