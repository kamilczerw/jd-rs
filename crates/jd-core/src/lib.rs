//! Core primitives for the Rust port of the `jd` JSON diff tool.
//!
//! The crate currently exposes the canonical data model used by the diff
//! engine together with helpers for parsing JSON/YAML inputs into the
//! canonical representation. Future milestones will extend this module with
//! diffing, patching, rendering, and canonicalization pipelines that mirror
//! the Go implementation.
//!
//! ```
//! use jd_core::{Node, DiffOptions};
//!
//! // Parse two JSON fragments and compare them structurally.
//! let lhs = Node::from_json_str("{\"name\": \"jd\", \"version\": 2}")?;
//! let rhs = Node::from_json_str("{\"version\": 2, \"name\": \"jd\"}")?;
//!
//! let opts = DiffOptions::default();
//! assert!(lhs.eq_with_options(&rhs, &opts));
//! # Ok::<(), jd_core::CanonicalizeError>(())
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod hash;
mod node;
mod number;
mod options;

pub use error::{CanonicalizeError, OptionsError};
pub use node::Node;
pub use number::Number;
pub use options::{ArrayMode, DiffOptions};

/// Returns the semantic version of the `jd-core` crate.
///
/// ```
/// assert!(!jd_core::version().is_empty());
/// ```
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
