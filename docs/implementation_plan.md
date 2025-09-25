# Implementation Plan for jd-rs Port

## Overview
This plan expands on the previously defined milestones and decomposes them into executable tasks with explicit dependencies, artifacts, and verification activities. The intent is to deliver a production-ready Rust port of the Go `jd` tool with full feature parity, robust testing, and comprehensive documentation.

## Milestone Breakdown

### 1. Workspace Bootstrap and Tooling Foundation (Day 1)
- **Tasks**
  - Initialize Cargo workspace structure with crates `jd-core`, `jd-cli`, `jd-benches`, and `jd-fuzz`.
  - Add workspace-level `Cargo.toml`, shared `rust-toolchain.toml`, `rustfmt.toml`, and `clippy.toml` configurations.
  - Configure linting gates in CI (GitHub Actions `ci.yml` stub) to run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`.
  - Introduce `.editorconfig` and base `.gitignore` entries for consistency.
- **Deliverables**
  - Compiling workspace with placeholder modules and binaries that build with `cargo check`.
  - Initial CI workflow executing successfully in local dry-run (using `act` or manual validation).
- **Verification**
  - `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all succeed.
  - CI workflow syntax validated via `act` or `yamllint`.

### 2. Core Data Model & Configuration Options (Days 2–3)
- **Tasks**
  - Define canonical `Node` enum and supporting `Number` representation ensuring parity with Go's `json.Number` behavior.
  - Implement conversion APIs: `Node::from_json_str`, `Node::from_yaml_str`, and `Node::to_value` for serde interoperability.
  - Introduce `DiffOptions`, `ArrayMode`, and validation to prevent conflicting settings.
  - Provide serde support for nodes to facilitate golden fixture serialization.
  - Create rustdoc-covered constructors, iterators, and equality semantics.
- **Deliverables**
  - `jd-core` library compiling with data model modules (`node.rs`, `number.rs`, `options.rs`).
  - Unit tests covering numeric edge cases (big ints, decimals) and equality semantics.
  - Rustdoc examples for every public function and type.
- **Verification**
  - `cargo test -p jd-core` includes doc tests (`cargo test --doc`).
  - Property tests verifying that converting to/from serde `Value` retains structure when possible.

### 3. Canonicalization Pipeline (Days 3–5)
- **Tasks**
  - Implement JSON parsing pipeline using streaming deserializers to avoid loading entire inputs when unnecessary.
  - Implement YAML parsing with tag and anchor preservation strategy; evaluate serde_yaml vs custom parser and document decision in ADR.
  - Normalize whitespace, ordering, and duplicate key handling to mirror Go implementation.
  - Introduce limiters for recursion depth and input size; expose configuration hooks.
  - Build property tests ensuring canonicalization invariants (idempotence, stable ordering).
- **Deliverables**
  - `parse` module with `Reader` abstractions for files, stdin, and in-memory buffers.
  - Tests verifying BOM stripping, CRLF handling, invalid JSON/YAML error parity (messages captured from Go golden outputs).
- **Verification**
  - Property tests using `proptest` for random JSON values.
  - CLI fixture that pipes through canonicalization-only command (temporary debug CLI subcommand or library test harness).

### 4. Diff Engine — List Mode MVP (Days 5–8)
- **Tasks**
  - Implement structural diff algorithm mirroring Go's behavior for objects and arrays in list mode (default).
  - Port or re-implement Myers LCS-based diff for arrays with deterministic tie-breaking.
  - Build `Diff` representation containing operations, path segments, and metadata required for rendering.
  - Generate golden fixtures using Go `jd` for a curated set of JSON documents.
- **Deliverables**
  - `diff` module with unit tests covering nested objects, mixed arrays, and primitive changes.
  - Golden tests comparing Rust diff output to Go diff output for baseline cases.
- **Verification**
  - CLI integration test `tests/cli/diff_basic.rs` passes using golden outputs.
  - Bench harness collects initial runtime metrics to establish baseline for later optimization.

### 5. Extended Array Semantics & Patch Data Model (Days 8–10)
- **Tasks**
  - Implement array modes: set semantics and key-indexed map semantics, ensuring deterministic canonicalization.
  - Add configuration parsing for CLI flags `-set`, `-list`, `-map` with mutual exclusivity enforcement.
  - Extend `Diff` data model to capture set/map operations; ensure raw format compatibility.
  - Build targeted fixtures for arrays in each mode, including duplicates and key collisions.
- **Deliverables**
  - Unit tests for array semantic toggles with parity goldens.
  - Updated documentation describing behavior and providing runnable examples.
- **Verification**
  - Property tests verifying that switching modes matches Go output for same fixtures (scripted parity check).

### 6. Patch Apply/Reverse & Format Renderers (Days 10–13)
- **Tasks**
  - Implement patch application engine capable of applying diff to base document and generating target document.
  - Provide reverse operation for `-test` flag parity, ensuring idempotence checks.
  - Implement renderers for native JD format (v1/v2), RFC 6902 JSON Patch, RFC 7386 Merge Patch, and raw structural debug format.
  - Ensure serialization ordering and whitespace matches Go outputs (use golden comparisons).
  - Document conversion APIs in rustdoc with runnable examples.
- **Deliverables**
  - `patch` and `render` modules complete with unit and integration tests.
  - CLI option wiring for `-patch`, `-merge`, and `-raw` returning expected outputs.
- **Verification**
  - Golden fixtures for each format produced by Go tool and matched byte-for-byte.
  - Property test verifying `apply_patch(diff(a,b), a) == b` and `apply_patch(diff(a,b), b)` fails gracefully.

### 7. CLI Parity & User Experience (Days 13–15)
- **Tasks**
  - Implement Clap-based CLI replicating flags, help text, version output, exit codes, and color handling (`-color`, `-nocolor`, `NO_COLOR`).
  - Add support for reading from files or stdin/stdout with `-` semantics and TTY detection for color auto-mode.
  - Wire `-test` flag to execute roundtrip validation and return exit status consistent with Go implementation.
  - Integrate logging under `--verbose` (if introduced) without altering default behavior.
- **Deliverables**
  - CLI crate ready with integration tests covering help text snapshot, version output, error handling, and exit codes.
- **Verification**
  - `assert_cmd`-based tests with golden outputs.
  - Manual parity harness run comparing Go vs Rust CLI for diverse fixtures.

### 8. Comprehensive Testing & Parity Automation (Days 15–18)
- **Tasks**
  - Flesh out integration test suite covering JSON, YAML, large files, edge cases (duplicate keys, invalid inputs).
  - Set up `cargo-fuzz` targets and add CI smoke job running short fuzzing sessions.
  - Implement `scripts/gen_golden_from_go.sh` and `scripts/parity_check.sh` to regenerate fixtures and verify parity routinely.
  - Configure code coverage tooling (grcov) with minimum thresholds enforced in CI.
- **Deliverables**
  - Extensive test matrix documented in repository with instructions for regeneration.
  - CI updates to execute parity script on Linux job and upload coverage reports.
- **Verification**
  - CI pipelines passing consistently.
  - Fuzz targets run without crashes for baseline corpora.

### 9. Performance Benchmarking & Optimization (Days 18–20)
- **Tasks**
  - Implement Criterion benchmarks in `jd-benches` for representative datasets (e.g., Kubernetes manifests, large JSON arrays, GitHub API responses).
  - Create `scripts/bench_vs_go.sh` to compare Rust vs Go runtimes and memory using `/usr/bin/time`.
  - Profile diff hotspots (using `cargo flamegraph` or `perf`) and optimize (e.g., caching hash computations, minimizing allocations) while maintaining determinism.
  - Document results and baseline metrics in `docs/benchmarks.md`.
- **Deliverables**
  - Benchmark suite committed with reproducible instructions and baseline numbers.
  - Optimization PR notes recorded in ADR if major algorithmic changes occur.
- **Verification**
  - Benchmarks show Rust implementation within ±5% of Go version or justified deviations with action items.

### 10. Documentation, Release Readiness, and Final QA (Days 20–22)
- **Tasks**
  - Ensure 100% rustdoc coverage with runnable examples (verify with `cargo deadlinks` and `cargo doc --document-private-items` as needed).
  - Author README (usage, installation, compatibility, benchmarks, roadmap) and supporting documents (`CONTRIBUTING`, `CODE_OF_CONDUCT`, `SECURITY`, `CHANGELOG`).
  - Finalize license headers and attribution to Go project.
  - Run full parity harness, fuzzing, coverage, and benchmarks; capture results for release notes.
  - Prepare release checklist and GitHub release template.
- **Deliverables**
  - Documentation suite committed and passing doc tests.
  - Release plan ready for tag `v0.1.0` (pre-release) with parity statement.
- **Verification**
  - `cargo doc --no-deps -D warnings` succeeds.
  - Final CI run green across matrix.

## Cross-Cutting Activities
- **Risk Tracking**: Update risk register after each milestone, capturing mitigations and status.
- **ADR Maintenance**: Record significant design decisions (number handling, YAML emitter choice, diff algorithm) immediately after milestones that introduce them.
- **Stakeholder Reviews**: Schedule review checkpoints after Milestones 4, 7, and 10 to validate direction and accept deliverables.
- **Dependency Hygiene**: Run `cargo update -p` and `cargo-deny` periodically to ensure dependency health and license compliance.

## Acceptance Criteria Summary
- Full CLI parity with Go `jd` including outputs, exit codes, and color behavior.
- Core library exposes documented APIs with runnable examples and zero panics on expected error paths.
- Automated test suite covering unit, integration, property, fuzz, and parity tests with coverage >= 85%.
- Benchmark suite demonstrating performance within ±5% of Go implementation on agreed datasets.
- Documentation and release collateral ready for public consumption, including parity statement and contribution guidelines.

## Timeline Snapshot
| Milestone | Target Days |
|-----------|-------------|
| 1 | Day 1 |
| 2 | Days 2–3 |
| 3 | Days 3–5 |
| 4 | Days 5–8 |
| 5 | Days 8–10 |
| 6 | Days 10–13 |
| 7 | Days 13–15 |
| 8 | Days 15–18 |
| 9 | Days 18–20 |
| 10 | Days 20–22 |

## Dependencies & Tooling Notes
- **Go `jd` Binary**: Required for generating goldens and parity checks; pin to v2.2.2.
- **External Crates**: Prefer stable, well-maintained crates (`serde`, `serde_json`, `serde_yaml`, `indexmap`, `clap`, `anyhow`, `thiserror`, `criterion`, `proptest`, `cargo-fuzz`, `anstream`).
- **System Tools**: `jq`, `diff`, `yamllint`, `grcov`, `perf`/`flamegraph`, and `Go` toolchain for parity scripts.

## Open Questions & Follow-Ups
- Confirm feasibility of matching Go's YAML anchor behavior using serde-based tooling; document findings in ADR 0003.
- Determine whether streaming diff should be exposed publicly or kept internal pending performance evaluation.
- Evaluate need for optional parallel diff feature flag; defer decision until after baseline benchmarks.

## Review & Maintenance
- Store this plan under version control (`docs/implementation_plan.md`) and update as milestones evolve.
- Revisit plan after each milestone for adjustments based on findings, recording changes in CHANGELOG or ADRs as appropriate.

