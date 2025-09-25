# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Comprehensive rustdoc coverage with runnable examples across `jd-core` public APIs.
- Expanded repository and crate READMEs with installation, usage, and parity details.
- Documentation collateral: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and `SECURITY.md`.
- Initial changelog entry tracking documentation milestone progress.
- Multi-platform CI/CD workflow covering fmt, clippy, tests, doc tests, docs build, cargo-deny, coverage floors, and Criterion-based performance guardrails.
- Criterion benchmark baseline (`crates/jd-benches/baselines/criterion-ci.json`) plus regression checker script (`scripts/check_bench_regressions.py`).
- Draft release notes for v0.1.0 summarising parity, coverage, benchmarks, and licensing.

### Changed
- Updated docs/architecture overview to reflect the current implementation state.
- Refreshed milestone status report for the documentation pass.
