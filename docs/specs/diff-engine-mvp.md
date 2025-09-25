# Diff Engine MVP Specification

## Scope
- Implement the structural diff for `ArrayMode::List` across all node variants in `jd-core`.
- Provide typed representations for diffs (`Diff`, `DiffElement`), metadata, and path segments compatible with Go `jd` v2.2.2 native format.
- Introduce internal helpers for Myers-style LCS alignment used by list diffs, including deterministic tie-breaking to match Go output ordering.
- Expose parsing/serialization shims sufficient for test fixtures (full renderers ship in later milestones).

## Module Layout
- `crates/jd-core/src/diff/mod.rs` – public entry points (`diff`, `Diff`, `DiffElement`), plus serde support for the native format used in goldens.
- `crates/jd-core/src/diff/path.rs` – `PathSegment` enum (Key, Index, Set, SetKeys placeholder), cloning helpers, equality, and formatting for debug/testing.
- `crates/jd-core/src/diff/list.rs` – list diff implementation with LCS, cursor walk, context capture, and recursion into child nodes.
- `crates/jd-core/src/diff/object.rs` – object diff implementation (lexical key ordering, recursion, add/remove hunks).
- `crates/jd-core/src/diff/primitives.rs` – fallbacks for scalar mismatches leveraging `Node::eq_with_options` and `Node::hash_code`.
- Future milestones will extend this tree with set/multiset support and renderers; keep APIs internal or `pub(crate)` when possible until stabilized via ADR.

## Data Structures
- `DiffMetadata` struct mirroring Go's `Metadata` (fields: `merge: bool`, `set_keys: Option<Vec<String>>`, `color: Option<bool>>` placeholder for future). Only `merge` is required for list-mode MVP.
- `DiffElement` struct containing:
  - `metadata: Option<DiffMetadata>` (only serialized when Some).
  - `path: Vec<PathSegment>`.
  - `before: Vec<Node>` and `after: Vec<Node>` (list context).
  - `remove: Vec<Node>` and `add: Vec<Node>`.
- `Diff` type aliasing `Vec<DiffElement>` with constructor helpers (`Diff::empty`, `Diff::singleton`).
- `PathSegment` enum supporting `Key(String)` and `Index(i64)` for MVP, plus `Set`, `SetKeys(BTreeMap<_, _>)`, etc., stubbed but unimplemented operations returning errors when used.

## Algorithm Details
### Common Flow
1. Entry point `Node::diff(&self, other, options: &DiffOptions) -> Diff` delegates based on enum variant.
2. If nodes are equal under options, return empty diff.
3. When variants differ, emit a replacement hunk containing `remove` (old) and `add` (new) arrays; list/object variants call specialized logic first.

### Primitive Nodes (`Void`, `Null`, `Bool`, `Number`, `String`)
- On mismatch, emit a single hunk with `path` being the current path, `remove` containing `self.clone()` (unless `Void`), and `add` containing `other.clone()`.
- `before`/`after` remain empty.

### Objects (`Node::Object`)
- Gather keys from both objects, sort lexicographically, and iterate once for removals (keys only in left) and additions (keys only in right).
- When key exists in both maps, recursively diff child nodes with path extended by `PathSegment::Key(key.clone())`.
- For keys missing from RHS, emit hunk with `remove=[old]`, `add=[]`.
- For keys missing from LHS, emit hunk with `remove=[]`, `add=[new]`.
- Merge metadata is hard-coded to `merge=false` for MVP; future toggles will set it based on `DiffOptions`.

### Arrays (`Node::Array`) – List Mode
1. Compute hash codes for each element using `Node::hash_code(options)`.
2. Run Myers LCS on the hash sequences; we will implement a deterministic LCS variant that favors earlier elements (mirroring Go's usage of `golcs`).
3. Walk the arrays with cursors `i` (lhs), `j` (rhs), `lcs_idx` (common sequence index), and maintain `path_cursor` representing the insertion index in diff path semantics.
4. Accumulate pending hunk data:
   - `before` seeded with the previous RHS element (or `Void` when at start).
   - `remove` appended when advancing lhs without matching LCS.
   - `add` appended when advancing rhs without matching LCS.
5. When encountering nested containers of compatible types, flush pending context (`after` = next lhs element or `Void`) before recursing and appending the nested diff to the result list.
6. When reaching end-of-array conditions, flush remaining `add` or `remove` entries accordingly and set `after` to trailing context if needed.
7. Recursively continue with remaining suffix by calling `diff_rest` equivalent until both arrays consumed.

### Path Semantics
- For list diffs, the first hunk path is parent path plus `PathSegment::Index(start_index)` representing the insertion point. Each recursion increments `path_cursor` consistent with Go's behavior (increments by 2 when only additions appended, by 1 otherwise).
- For object diffs, paths simply append `PathSegment::Key` for nested traversal.
- Provide helper to `clone_drop_last` mirroring Go's `path.clone().drop()` when building nested contexts.

## Testing Strategy
- **Unit tests** (`crates/jd-core/src/diff/...`) covering:
  - Replacement hunks for scalar mismatches.
  - Object diff scenarios: key removal, addition, nested recursion.
  - Array diff cases: pure insert, pure delete, substitution, nested object diff inline.
- **Property tests** verifying that diff is empty when nodes equal and that `diff` respects array equality invariants.
- **Golden tests** under `crates/jd-core/tests/fixtures/diff/list/` comparing Rust-rendered diffs (serialized JSON) against outputs generated by Go `jd` for curated fixtures (simple arrays, nested objects, multiple operations).
- **CLI integration** (`tests/cli/diff_basic.rs`) to invoke `jd-cli diff` with fixtures and compare stdout/stderr/exit code to golden expectations.

## Dependencies
- Introduce a small, `no_std`-friendly Myers LCS helper under `jd-core` rather than external crates to keep control over determinism.
- No new external dependencies required for MVP; reuse existing serde/test stacks.

## Out of Scope / Deferred
- Set and multiset modes, metadata flags (`set`, `mset`, `setkeys`).
- Rendering diffs into CLI formats (native, JSON Patch, etc.).
- Patch application logic, merge semantics toggles.
- Performance tuning beyond straightforward iteration.

## Open Questions
- Validate whether Go's `golcs` tie-breaking always prefers earliest matches; confirm by running targeted fixtures when generating goldens. If discrepancies arise, document in ADR.
