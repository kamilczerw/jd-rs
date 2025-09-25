//! Benchmark harness scaffolding for the Rust port of the `jd` tool.
//!
//! The crate currently exposes a trivial helper so doctests can run
//! while the real benchmarks are authored in a later milestone.
//!
//! # Examples
//!
//! ```
//! assert_eq!(jd_benches::is_ready(), false);
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Indicates whether Criterion benchmarks have been added.
///
/// Returns `false` until the performance milestone introduces real
/// benchmark groups.
pub fn is_ready() -> bool {
    false
}
