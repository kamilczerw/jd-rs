# 0002 — Adopt `arbitrary` for fuzz harness generation

## Status
Accepted

## Context
Milestone 5 requires standing up fuzz targets for canonicalization, diffing, and
patch application as part of the Rust workspace's parity plan. The harnesses
need to deterministically derive structured JSON inputs from raw byte streams so
`cargo fuzz` can exercise deep parser and patch code paths without tripping over
UTF-8 or schema expectations. Hand-rolling this translation with ad-hoc splits
or UTF-8 assumptions produced extremely low coverage in local experiments.

## Decision
Add the well-supported [`arbitrary`](https://crates.io/crates/arbitrary) crate as
a dependency of `jd-fuzz` (and the standalone `cargo fuzz` package) to interpret
fuzzer-provided bytes into bounded JSON structures. The helper functions reuse
`Unstructured` to generate nested arrays/objects while respecting the plan's
size limits, ensuring stable coverage and avoiding panics.

## Alternatives Considered
- **Parse bytes as JSON strings directly:** Rejected because random data rarely
  forms valid JSON, yielding ineffective fuzzing.
- **Reimplement a bespoke generator:** Rejected due to higher maintenance cost
  and increased risk of diverging from the upstream Go semantics.
- **Depend on property-test generators (`proptest`):** Rejected because
  `proptest`'s runner expects control over randomness and does not integrate
  cleanly with `libFuzzer` byte streams.

## Consequences
- `jd-fuzz` and the `cargo fuzz` harness depend on `arbitrary`, which is already
  widely used in the fuzzing ecosystem and maintained for the current Rust
  release cadence.
- The shared helper functions can now be reused by both fuzz targets and future
  smoke tests, improving coverage without duplicating generators.
- CI gains a lightweight nightly fuzz job that leverages these helpers while
  keeping the main test job fast.
