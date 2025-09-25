# Patch Rendering & Reverse Operations Specification

## Scope
- Extend `jd-core` with rendering support for the native jd diff format, RFC 6902 JSON Patch, RFC 7386 JSON Merge Patch, and a raw JSON debug view.
- Provide an explicit `Diff::reverse` helper mirroring Go patch semantics for strict-mode diffs so higher layers can validate round-trips.
- Expose ergonomic render configuration describing color output without exposing internal metadata toggles.

## Goals
- Match Go `jd` v2.2.2 output byte-for-byte for all supported renderings.
- Preserve metadata inheritance semantics when rendering or reversing diffs.
- Surface descriptive errors for unsupported transformations (e.g., reversing a merge diff, rendering invalid list contexts into JSON Patch).
- Keep APIs fully documented with runnable doctests and minimal allocations.

## Non-Goals
- Implement set/multiset aware renderers (deferred to milestones covering set semantics).
- Recreate the Go option-style variadic API; we expose a strongly typed Rust configuration instead.
- Provide streaming readers for patch/merge inputs; parsing will ship alongside CLI integration.

## API Surface
- `pub struct RenderConfig { pub color: bool }` with `Default` (color disabled) and builder-style helpers.
- `pub enum RenderError` with variants for unsupported merges, invalid contexts, pointer failures, and serialization issues.
- `impl Diff` additions:
  - `pub fn render(&self, config: &RenderConfig) -> String` – native jd text (supports color when `config.color`).
  - `pub fn render_patch(&self) -> Result<String, RenderError>` – strict-mode JSON Patch.
  - `pub fn render_merge(&self) -> Result<String, RenderError>` – merge patch serialization via patch engine (requires merge metadata).
  - `pub fn render_raw(&self) -> Result<String, RenderError>` – `serde_json` dump of the diff structure for debugging.
  - `pub fn reverse(&self) -> Result<Diff, RenderError>` – swap additions/removals for strict diffs while validating metadata inheritance.
- `impl DiffElement` helper `fn render_native(&self, config: &RenderConfig, inherited: &DiffMetadata) -> String` (module-private).
- `impl DiffMetadata { pub fn render_header(&self) -> String }` mirroring Go metadata lines.
- `impl Path { pub fn to_node(&self) -> Node; pub fn to_pointer(&self) -> Result<String, RenderError> }` with JSON pointer escaping rules.

## Design Notes
- Native renderer replicates Go's line-based format, including `[` / `]` sentinels for void context and colorized single-string diffs by computing a character-level LCS.
- JSON Patch renderer enforces the same restrictions as Go (single line of before/after context, prohibiting "numeric-looking" object keys) and generates `test/remove/add` sequences.
- Merge renderer first validates that every element (including inherited metadata) is merge-enabled, coerces `Void` additions to `Null`, then patches against `Node::Void` to reuse the canonicalization pipeline.
- `reverse` walks elements while tracking inherited metadata. Encountering merge metadata yields an error because original values are not preserved in merge diffs.
- Raw renderer serializes via `serde_json::to_string` to ease golden generation in future milestones.

## Testing Strategy
- Unit tests comparing native render output for representative diffs (object change, list substitution, string diff with colors disabled/enabled).
- JSON Patch golden-style assertions for array substitution verifying context tests, and failure cases (multiple before context, empty add/remove) returning specific errors.
- Merge renderer success case using a manually constructed merge diff and failure when encountering strict hunks.
- Property test exercising `reverse`: for random JSON pairs under list semantics, ensure `apply_patch(diff.reverse(), target) == base`.
- Doctests for each public API demonstrating usage and expected strings.

## Risks & Mitigations
- **Pointer escaping**: implement dedicated helper mirroring Go's logic (reject numeric-looking keys, escape `~` and `/`). Add unit tests for edge cases.
- **Color diff accuracy**: reuse deterministic LCS routine for rune sequences; add targeted test covering unicode input.
- **Merge reversibility**: document in rustdoc and errors that merge diffs cannot be reversed; guard via metadata traversal.

## Dependencies
- No new external crates required; rely on `serde_json` for serialization and internal helpers.
