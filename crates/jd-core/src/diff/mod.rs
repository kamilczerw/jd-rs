//! Diff data structures and algorithms.
//!
//! The module defines the native diff representation used by `jd-core` along
//! with helper utilities for constructing, iterating, and serializing diffs.
//! The current milestone implements list-mode diffing and object traversal,
//! mirroring the upstream Go implementation.

mod list;
mod object;
mod path;
mod primitives;

pub use path::{path_from_segments, root_path, Path, PathSegment};

use serde::{Deserialize, Serialize};

use crate::{ArrayMode, DiffOptions, Node};

/// Metadata associated with a diff element.
///
/// ```
/// # use jd_core::diff::DiffMetadata;
/// let meta = DiffMetadata::merge();
/// assert!(meta.merge);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffMetadata {
    /// Indicates that merge patch semantics should be used.
    #[serde(default)]
    pub merge: bool,
}

impl DiffMetadata {
    /// Constructs metadata for merge mode.
    #[must_use]
    pub fn merge() -> Self {
        Self { merge: true }
    }
}

/// Represents a single diff hunk.
///
/// ```
/// # use jd_core::diff::{DiffElement, PathSegment};
/// # use jd_core::{Node, DiffOptions};
/// let lhs = Node::from_json_str("1").unwrap();
/// let rhs = Node::from_json_str("2").unwrap();
/// let element = DiffElement::new()
///     .with_path(vec![])
///     .with_remove(vec![lhs.clone()])
///     .with_add(vec![rhs.clone()]);
/// assert_eq!(element.remove, vec![lhs.clone()]);
/// assert_eq!(element.add, vec![rhs.clone()]);
/// # let diff = lhs.diff(&rhs, &DiffOptions::default());
/// # assert_eq!(diff.len(), 1);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DiffElement {
    /// Optional metadata for this hunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<DiffMetadata>,
    /// JSON Pointer-like path to the affected location.
    #[serde(default)]
    pub path: Path,
    /// Context before the change (list diffs only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub before: Vec<Node>,
    /// Values removed at the path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remove: Vec<Node>,
    /// Values added at the path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add: Vec<Node>,
    /// Context after the change (list diffs only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub after: Vec<Node>,
}

impl DiffElement {
    /// Creates a blank diff element.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the metadata for the element.
    #[must_use]
    pub fn with_metadata(mut self, metadata: DiffMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets the path for the element.
    #[must_use]
    pub fn with_path<P>(mut self, path: P) -> Self
    where
        P: Into<Path>,
    {
        self.path = path.into();
        self
    }

    /// Sets the before context.
    #[must_use]
    pub fn with_before(mut self, before: Vec<Node>) -> Self {
        self.before = before;
        self
    }

    /// Sets the removal list.
    #[must_use]
    pub fn with_remove(mut self, remove: Vec<Node>) -> Self {
        self.remove = remove;
        self
    }

    /// Sets the addition list.
    #[must_use]
    pub fn with_add(mut self, add: Vec<Node>) -> Self {
        self.add = add;
        self
    }

    /// Sets the after context.
    #[must_use]
    pub fn with_after(mut self, after: Vec<Node>) -> Self {
        self.after = after;
        self
    }
}

/// Collection of diff elements.
///
/// ```
/// # use jd_core::diff::{Diff, DiffElement};
/// let diff = Diff::from_elements(vec![DiffElement::new()]);
/// assert_eq!(diff.len(), 1);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Diff {
    elements: Vec<DiffElement>,
}

impl Diff {
    /// Constructs an empty diff.
    #[must_use]
    pub fn empty() -> Self {
        Self { elements: Vec::new() }
    }

    /// Builds a diff from the provided elements.
    #[must_use]
    pub fn from_elements(elements: Vec<DiffElement>) -> Self {
        Self { elements }
    }

    /// Returns the number of elements in the diff.
    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Indicates whether the diff is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns an iterator over the elements.
    pub fn iter(&self) -> std::slice::Iter<'_, DiffElement> {
        self.elements.iter()
    }

    /// Consumes the diff and returns the elements.
    #[must_use]
    pub fn into_elements(self) -> Vec<DiffElement> {
        self.elements
    }
}

impl IntoIterator for Diff {
    type Item = DiffElement;
    type IntoIter = std::vec::IntoIter<DiffElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a> IntoIterator for &'a Diff {
    type Item = &'a DiffElement;
    type IntoIter = std::slice::Iter<'a, DiffElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl From<Vec<DiffElement>> for Diff {
    fn from(value: Vec<DiffElement>) -> Self {
        Self::from_elements(value)
    }
}

/// Computes the structural diff between two nodes.
#[must_use]
pub fn diff_nodes(lhs: &Node, rhs: &Node, options: &DiffOptions) -> Diff {
    diff_impl(lhs, rhs, &Path::new(), options)
}

pub(super) fn diff_impl(lhs: &Node, rhs: &Node, path: &Path, options: &DiffOptions) -> Diff {
    if lhs.eq_with_options(rhs, options) {
        return Diff::empty();
    }

    match (lhs, rhs) {
        (Node::Object(left), Node::Object(right)) => {
            object::diff_objects(left, right, path, options)
        }
        (Node::Array(left), Node::Array(right)) => match options.array_mode() {
            ArrayMode::List => list::diff_lists(left, right, path, options),
            mode => {
                panic!("array mode {mode:?} not implemented in diff engine");
            }
        },
        _ => primitives::diff_primitives(lhs, rhs, path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DiffOptions;
    use proptest::prelude::*;

    #[test]
    fn diff_of_numbers_produces_replacement_hunk() {
        let lhs = Node::from_json_str("1").unwrap();
        let rhs = Node::from_json_str("2").unwrap();
        let diff = diff_nodes(&lhs, &rhs, &DiffOptions::default());
        let expected = Diff::from_elements(vec![DiffElement::new()
            .with_path(Path::new())
            .with_remove(vec![lhs])
            .with_add(vec![rhs])]);
        assert_eq!(diff, expected);
    }

    #[test]
    fn diff_of_objects_tracks_additions_and_removals() {
        let lhs = Node::from_json_str("{\"a\":1,\"b\":2}").unwrap();
        let rhs = Node::from_json_str("{\"b\":2,\"c\":3}").unwrap();
        let diff = diff_nodes(&lhs, &rhs, &DiffOptions::default());
        let expected = Diff::from_elements(vec![
            DiffElement::new()
                .with_path(PathSegment::key("a"))
                .with_remove(vec![Node::from_json_str("1").unwrap()]),
            DiffElement::new()
                .with_path(PathSegment::key("c"))
                .with_add(vec![Node::from_json_str("3").unwrap()]),
        ]);
        assert_eq!(diff, expected);
    }

    #[test]
    fn diff_of_arrays_with_substitution_preserves_context() {
        let lhs = Node::from_json_str("[1,2,3]").unwrap();
        let rhs = Node::from_json_str("[1,4,3]").unwrap();
        let diff = diff_nodes(&lhs, &rhs, &DiffOptions::default());
        let expected = Diff::from_elements(vec![DiffElement::new()
            .with_path(Path::from(vec![PathSegment::index(1)]))
            .with_before(vec![Node::from_json_str("1").unwrap()])
            .with_remove(vec![Node::from_json_str("2").unwrap()])
            .with_add(vec![Node::from_json_str("4").unwrap()])
            .with_after(vec![Node::from_json_str("3").unwrap()])]);
        assert_eq!(diff, expected);
    }

    #[test]
    fn diff_of_arrays_with_append_marks_void_context() {
        let lhs = Node::from_json_str("[1,2]").unwrap();
        let rhs = Node::from_json_str("[1,2,3]").unwrap();
        let diff = diff_nodes(&lhs, &rhs, &DiffOptions::default());
        let expected = Diff::from_elements(vec![DiffElement::new()
            .with_path(Path::from(vec![PathSegment::index(2)]))
            .with_before(vec![Node::from_json_str("2").unwrap()])
            .with_add(vec![Node::from_json_str("3").unwrap()])
            .with_after(vec![Node::Void])]);
        assert_eq!(diff, expected);
    }

    fn arb_json_value() -> impl Strategy<Value = serde_json::Value> {
        use proptest::{collection::btree_map, collection::vec, string::string_regex};

        let leaf = prop_oneof![
            Just(serde_json::Value::Null),
            any::<bool>().prop_map(serde_json::Value::Bool),
            proptest::num::f64::ANY.prop_filter_map("finite", |f| {
                if f.is_finite() {
                    serde_json::Number::from_f64(f).map(serde_json::Value::Number)
                } else {
                    None
                }
            }),
            string_regex("[a-zA-Z0-9]{0,8}").unwrap().prop_map(serde_json::Value::String),
        ];
        leaf.prop_recursive(4, 8, 4, move |inner| {
            prop_oneof![
                vec(inner.clone(), 0..4).prop_map(serde_json::Value::Array),
                btree_map(string_regex("[a-zA-Z0-9]{1,8}").unwrap(), inner, 0..4).prop_map(|map| {
                    let mut object = serde_json::Map::new();
                    for (k, v) in map {
                        object.insert(k, v);
                    }
                    serde_json::Value::Object(object)
                }),
            ]
        })
    }

    proptest! {
        #[test]
        fn identical_nodes_produce_empty_diff(json in arb_json_value()) {
            let node = Node::from_json_value(json.clone()).unwrap();
            let other = Node::from_json_value(json).unwrap();
            let diff = diff_nodes(&node, &other, &DiffOptions::default());
            prop_assert!(diff.is_empty());
        }
    }
}
