//! Benchmark harness support for the Rust port of the `jd` tool.
//!
//! The crate exposes curated benchmark corpora that mirror the
//! real-world payloads we use to compare the Rust implementation
//! against the upstream Go binary. Criterion benchmarks (see
//! `benches/`) consume these corpora directly so that all
//! microbenchmarks, parity harnesses, and documentation examples
//! operate on the same data.
//!
//! # Examples
//!
//! ```
//! use jd_benches::available_corpora;
//! use jd_core::DiffOptions;
//!
//! let corpus = available_corpora()
//!     .iter()
//!     .find(|c| c.name() == "kubernetes-deployment")
//!     .expect("corpus registered");
//! let dataset = corpus.load().expect("fixtures parse");
//! let diff = dataset.diff(&DiffOptions::default());
//! assert!(!diff.is_empty());
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use jd_core::{CanonicalizeError, Diff, DiffOptions, Node, RenderConfig};

const KUBERNETES_BEFORE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/kubernetes/before.json"));
const KUBERNETES_AFTER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/kubernetes/after.json"));
const GITHUB_BEFORE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/github/before.json"));
const GITHUB_AFTER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/github/after.json"));
const LARGE_ARRAY_BEFORE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/large-array/before.json"));
const LARGE_ARRAY_AFTER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/large-array/after.json"));

/// Identifies a benchmark corpus backed by JSON fixtures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Corpus {
    name: &'static str,
    description: &'static str,
    before: &'static str,
    after: &'static str,
}

impl Corpus {
    /// Creates a new corpus definition.
    const fn new(
        name: &'static str,
        description: &'static str,
        before: &'static str,
        after: &'static str,
    ) -> Self {
        Self { name, description, before, after }
    }

    /// Returns the short identifier used for benchmark labels.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Human-readable summary of what the dataset represents.
    #[must_use]
    pub fn description(&self) -> &'static str {
        self.description
    }

    /// Returns the total size in bytes of the source fixtures.
    #[must_use]
    pub fn fixture_bytes(&self) -> usize {
        self.before.len() + self.after.len()
    }

    /// Loads the corpus into canonical `Node` instances.
    ///
    /// ```
    /// use jd_benches::available_corpora;
    /// use jd_core::DiffOptions;
    ///
    /// let corpus = available_corpora()
    ///     .iter()
    ///     .find(|c| c.name() == "github-issue")
    ///     .unwrap();
    /// let dataset = corpus.load().unwrap();
    /// let diff = dataset.diff(&DiffOptions::default());
    /// assert!(!diff.is_empty());
    /// ```
    pub fn load(&self) -> Result<Dataset, CanonicalizeError> {
        Ok(Dataset {
            before: Node::from_json_str(self.before)?,
            after: Node::from_json_str(self.after)?,
        })
    }
}

/// Materialized benchmark dataset.
#[derive(Clone, Debug)]
pub struct Dataset {
    before: Node,
    after: Node,
}

impl Dataset {
    /// Returns the canonicalized "before" document.
    #[must_use]
    pub fn before(&self) -> &Node {
        &self.before
    }

    /// Returns the canonicalized "after" document.
    #[must_use]
    pub fn after(&self) -> &Node {
        &self.after
    }

    /// Computes the diff between `before` and `after` using the provided options.
    ///
    /// ```
    /// use jd_benches::available_corpora;
    /// use jd_core::DiffOptions;
    ///
    /// let corpus = &available_corpora()[0];
    /// let dataset = corpus.load().unwrap();
    /// let diff = dataset.diff(&DiffOptions::default());
    /// assert_eq!(diff.is_empty(), false);
    /// ```
    #[must_use]
    pub fn diff(&self, options: &DiffOptions) -> Diff {
        self.before.diff(&self.after, options)
    }

    /// Renders the dataset diff using the native jd text format.
    ///
    /// ```
    /// use jd_benches::available_corpora;
    /// use jd_core::{DiffOptions, RenderConfig};
    ///
    /// let corpus = &available_corpora()[0];
    /// let dataset = corpus.load().unwrap();
    /// let diff = dataset.diff(&DiffOptions::default());
    /// let rendered = dataset.render_native(&diff, &RenderConfig::default());
    /// assert!(rendered.contains("@"));
    /// ```
    #[must_use]
    pub fn render_native(&self, diff: &Diff, config: &RenderConfig) -> String {
        diff.render(config)
    }
}

const CORPORA: &[Corpus] = &[
    Corpus::new(
        "kubernetes-deployment",
        "Rolling update of a Kubernetes Deployment manifest.",
        KUBERNETES_BEFORE,
        KUBERNETES_AFTER,
    ),
    Corpus::new(
        "github-issue",
        "Lifecycle update of a GitHub issue webhook payload.",
        GITHUB_BEFORE,
        GITHUB_AFTER,
    ),
    Corpus::new(
        "large-array",
        "Synthetic array workload exercising hashing and LCS traversal.",
        LARGE_ARRAY_BEFORE,
        LARGE_ARRAY_AFTER,
    ),
];

/// Returns the registered benchmark corpora.
#[must_use]
pub fn available_corpora() -> &'static [Corpus] {
    CORPORA
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpora_are_non_empty() {
        assert!(!available_corpora().is_empty());
        for corpus in available_corpora() {
            let dataset = corpus.load().expect("fixture parse");
            let diff = dataset.diff(&DiffOptions::default());
            assert!(!diff.is_empty(), "{} should produce a diff", corpus.name());
        }
    }
}
