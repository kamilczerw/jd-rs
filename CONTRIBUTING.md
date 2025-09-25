# Contributing to jd-rs

Thank you for your interest in helping port the Go [`jd`](https://github.com/josephburnett/jd) tool to Rust. This document outlines the expectations for feature work, bug fixes, and documentation improvements.

## Code of Conduct

Participation in this project is governed by the [Code of Conduct](CODE_OF_CONDUCT.md). By contributing you agree to uphold its principles.

## Development Workflow

1. **Sync the plan** – Re-read the milestone plan under [`docs/implementation_plan.md`](docs/implementation_plan.md) and the latest status update before starting work.
2. **Design first** – Draft component specifications and tests before implementing features. Non-trivial decisions must be captured in a new ADR (`ADRs/NNNN-*.md`).
3. **Keep parity** – When in doubt, inspect the Go `jd` repository (tag `v2.2.2`) and codify the observed behavior in tests before modifying Rust code.
4. **Tests before code** – Add unit, integration, property-based, fuzz, and parity tests as described in [`docs/specs`](docs/specs/). When fixing bugs, first reproduce them with a failing test.
5. **Document everything** – All public APIs require rustdoc comments with runnable examples. README and `docs/` examples must have matching tests or doctests.
6. **Validate locally** – Run the quality gates listed below before opening a pull request. Fix clippy, rustfmt, and cargo-deny issues immediately.

## Local Checks

Run the following commands from the repository root:

```console
$ cargo fmt --all
$ cargo clippy --all-targets --all-features -- -D warnings
$ cargo test --all --all-features
$ cargo test --doc --all
$ cargo deny check
```

For changes touching fuzzing or benchmarks, also run:

```console
$ cargo fuzz run canonicalization -- -max_total_time=30
$ cargo bench -p jd-benches
```

Record any deviations or failures (with justification) in an ADR before submitting patches.

## Pull Request Guidelines

- Keep commits focused and include descriptive messages.
- Update documentation (`README`, `docs/`, rustdoc) to reflect behavior changes.
- Regenerate golden fixtures with `scripts/gen_golden_from_go.sh` when parity expectations change.
- Reference relevant ADRs and link to upstream Go source lines in the PR description when explaining design choices.
- Ensure `docs/status.md` receives an updated milestone summary when advancing to the next phase.

## Licensing

By contributing to jd-rs you agree that your contributions will be licensed under the same terms as the rest of the project (see `LICENSE`).
