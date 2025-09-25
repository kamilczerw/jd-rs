//! Core library for the Rust port of the `jd` JSON diff tool.
//!
//! This crate will eventually expose the data model, canonicalization
//! pipeline, diff engine, and patch application APIs that mirror the Go
//! implementation. During the workspace scaffolding milestone it only
//! provides version metadata to enable smoke tests and doctest coverage.
//!
//! # Examples
//!
//! ```
//! use jd_core::version;
//!
//! assert!(version().starts_with('0'));
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Returns the semantic version of the `jd-core` crate.
///
/// The value is sourced from the crate metadata at compile time. This
/// helper keeps doctests alive while the rest of the surface is still
/// under construction.
///
/// ```
/// assert!(!jd_core::version().is_empty());
/// ```
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
