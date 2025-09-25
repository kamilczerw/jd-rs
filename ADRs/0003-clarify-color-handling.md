# 0003 — Clarify CLI Color Handling Parity

## Status
Accepted

## Context
The implementation plan for Milestone 7 calls for supporting `-color`, `-nocolor`, and the `NO_COLOR` environment variable as part of the CLI parity work.【e9580e†L89-L93】 The upstream Go CLI (v2.2.2), however, only exposes a `-color` boolean flag; it neither defines a `-nocolor` counterpart nor inspects `NO_COLOR` or TTY state before rendering diffs.【a69894†L18-L111】【765c3b†L229-L283】 Enabling extra toggles in the Rust port would diverge from the parity guardrail.

## Decision
Implement the Rust CLI to match Go behavior exactly: only honor an explicit `-color` flag to request ANSI styling, defaulting to plain output otherwise. Defer any additional toggles (`-nocolor`, `NO_COLOR`, auto TTY detection) unless upstream introduces them or a future ADR authorizes parity-breaking UX changes.

## Alternatives Considered
- **Add `-nocolor`/`NO_COLOR` support anyway:** Rejected because it would add new behaviors not present upstream, violating parity and increasing maintenance burden for undocumented options.
- **Auto-detect terminal capabilities:** Rejected for the same parity reasons; Go always emits plain text unless `-color` is explicitly set.

## Consequences
- Tests and documentation will describe color behavior exactly as upstream, simplifying cross-implementation parity checks.
- Future enhancements to color handling require explicit ADRs referencing upstream changes or a conscious divergence decision.

## References
- Implementation plan CLI task list.【e9580e†L89-L96】
- Go CLI flag definitions and render option wiring.【a69894†L18-L111】【765c3b†L229-L283】
