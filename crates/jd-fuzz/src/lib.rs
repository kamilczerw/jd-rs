//! Fuzzing harness placeholders for the Rust port of the `jd` tool.
//!
//! This crate will host `cargo-fuzz` entry points in a later milestone.
//! For now it exposes a simple function so that doctests can execute and
//! the crate participates in workspace builds.
//!
//! # Examples
//!
//! ```
//! assert_eq!(jd_fuzz::is_ready(), false);
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Indicates whether fuzz targets have been implemented.
///
/// The current milestone focuses on scaffolding, so this returns `false`
/// until dedicated fuzz targets are added.
pub fn is_ready() -> bool {
    false
}
