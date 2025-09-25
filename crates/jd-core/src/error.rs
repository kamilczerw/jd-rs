use thiserror::Error;

/// Errors that can occur while canonicalizing external data into [`Node`].
#[derive(Debug, Error)]
pub enum CanonicalizeError {
    /// The provided JSON input was invalid.
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// The provided YAML input was invalid.
    #[error("invalid YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    /// Encountered a number that cannot be represented as an IEEE-754 f64.
    #[error("number {value} cannot be represented as f64")]
    NumberOutOfRange {
        /// The textual representation of the offending number.
        value: String,
    },
    /// YAML maps may only contain string keys.
    #[error("unsupported YAML key type: {found}")]
    NonStringYamlKey {
        /// A description of the key that triggered the error.
        found: String,
    },
    /// YAML tags are not supported by the Go implementation and therefore
    /// rejected by the Rust port as well.
    #[error("unsupported YAML tag: {tag}")]
    UnsupportedYamlTag {
        /// The tag identifier encountered in the document.
        tag: String,
    },
    /// Attempted to construct a [`Number`] that is not finite.
    #[error("non-finite number encountered: {value}")]
    NotFinite {
        /// The offending numeric value.
        value: f64,
    },
}

/// Errors emitted when constructing [`DiffOptions`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum OptionsError {
    /// Precision tolerance is incompatible with set or multiset semantics.
    #[error("precision tolerance cannot be combined with set or multiset array modes")]
    PrecisionIncompatible,
    /// Set keys require arrays to operate in set mode.
    #[error("set keys require array mode to be set")]
    SetKeysRequireSetMode,
    /// Set keys must be non-empty strings.
    #[error("set keys must be non-empty strings")]
    EmptySetKey,
}
