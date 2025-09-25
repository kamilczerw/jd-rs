//! Core primitives for the Rust port of the `jd` JSON diff tool.
//!
//! `jd-core` mirrors the Go implementation's canonicalization, diff,
//! patch, and rendering semantics while exposing a convenient Rust API.
//! All public items include runnable examples to make parity expectations
//! explicit.
//!
//! ```
//! use jd_core::{DiffOptions, Node, RenderConfig};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let base = Node::from_json_str("{\"name\":\"jd\",\"version\":1}")?;
//!     let target = Node::from_json_str("{\"name\":\"jd\",\"version\":2}")?;
//!     let diff = base.diff(&target, &DiffOptions::default());
//!     assert!(!diff.is_empty());
//!
//!     let rendered = diff.render(&RenderConfig::default());
//!     assert!(rendered.contains("@ [\"version\"]"));
//!
//!     let patched = base.apply_patch(&diff)?;
//!     assert_eq!(patched, target);
//!     Ok(())
//! }
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod diff;
mod error;
mod hash;
mod node;
mod number;
mod options;
mod patch;

pub use diff::{Diff, DiffElement, DiffMetadata, Path, PathSegment, RenderConfig, RenderError};
pub use error::{CanonicalizeError, OptionsError};
pub use hash::{combine, hash_bytes, HashCode};
pub use node::Node;
pub use number::Number;
pub use options::{ArrayMode, DiffOptions};
pub use patch::PatchError;

/// Returns the semantic version of the `jd-core` crate.
///
/// ```
/// assert!(!jd_core::version().is_empty());
/// ```
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
