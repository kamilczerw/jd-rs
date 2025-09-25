# 0001 — Clarify `-test` Flag Expectation

## Status
Accepted

## Context
The implementation plan (Milestone 6) calls for "reverse operation for `-test` flag parity" and later wiring a CLI `-test` flag. After re-reading the upstream Go CLI (`v2.2.2`), there is no `-test` option—flags are limited to the set enumerated in `main.go` (`-color`, `-f`, `-git-diff-driver`, `-mset`, `-o`, `-p`, `-port`, `-precision`, `-set`, `-setkeys`, `-t`, `-version`, `-yaml`, `-v2`).【074fa5†L23-L199】 The only `-test` usage in the repository refers to Go's testing harness (e.g., `-test.run`) rather than user-facing CLI behavior. The milestone requirement therefore conflicts with the parity constraint.

## Decision
Treat the plan's reference to a `-test` flag as an internal shorthand for validating patch reversibility rather than a public CLI flag. We will:
- Implement `Diff::reverse` within `jd-core` to support automated round-trip validation.
- Keep the CLI surface aligned with Go (`jd` will not expose a `-test` flag unless upstream adds one).
- Document this interpretation in milestone status updates to avoid future confusion.

## Alternatives Considered
- **Implement a new `-test` flag anyway:** Rejected because it would violate the parity guardrail and add unsupported behavior.
- **Ignore reverse functionality entirely:** Rejected; the milestone still benefits from a library-level reverse helper used by tests and potential future parity scripts.

## Consequences
- Subsequent milestones can rely on `Diff::reverse` for invariants without exposing new CLI options.
- Any future upstream changes introducing a real `-test` flag will require revisiting this ADR.

## References
- Go CLI flag definitions in `v2.2.2/v2/jd/main.go`.【074fa5†L23-L199】
- Implementation plan requirement in `docs/implementation_plan.md`.【4be4c3†L1-L2】
