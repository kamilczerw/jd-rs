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
use serde_json::{self, Number as JsonNumber, Value as JsonValue};

use crate::{ArrayMode, DiffOptions, Node, PatchError};

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
    /// Optional set-key metadata (unused in list-mode MVP).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set_keys: Option<Vec<String>>,
    /// Optional color rendering hint (reserved for future parity work).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<bool>,
}

impl DiffMetadata {
    /// Constructs metadata for merge mode.
    #[must_use]
    pub fn merge() -> Self {
        Self { merge: true, set_keys: None, color: None }
    }

    pub(crate) fn is_effective(&self) -> bool {
        self.merge || self.set_keys.is_some() || self.color.is_some()
    }

    pub(crate) fn absorb(&mut self, other: &Self) {
        if other.merge {
            self.merge = true;
        }
        if let Some(keys) = &other.set_keys {
            self.set_keys = Some(keys.clone());
        }
        if let Some(color) = other.color {
            self.color = Some(color);
        }
    }

    fn render_header(&self) -> String {
        if self.merge {
            "^ {\"Merge\":true}\n".to_string()
        } else {
            String::new()
        }
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

/// Configuration toggles for diff rendering.
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderConfig {
    color: bool,
}

impl RenderConfig {
    /// Constructs a configuration with default settings (no ANSI color).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables ANSI color output.
    #[must_use]
    pub fn with_color(mut self, enabled: bool) -> Self {
        self.color = enabled;
        self
    }

    /// Indicates whether color output is enabled.
    #[must_use]
    pub fn color_enabled(self) -> bool {
        self.color
    }
}

impl RenderConfig {
    /// Convenience constructor enabling color output.
    #[must_use]
    pub fn color(enabled: bool) -> Self {
        Self::new().with_color(enabled)
    }
}

/// Errors that can occur while rendering or reversing diffs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderError {
    message: String,
}

impl RenderError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RenderError {}

impl From<serde_json::Error> for RenderError {
    fn from(err: serde_json::Error) -> Self {
        Self::new(err.to_string())
    }
}

impl From<PatchError> for RenderError {
    fn from(err: PatchError) -> Self {
        Self::new(err.to_string())
    }
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

    /// Renders the diff using the native jd text format.
    ///
    /// ```
    /// # use jd_core::{DiffOptions, Node, RenderConfig};
    /// let lhs = Node::from_json_str("{\"a\":1}")?;
    /// let rhs = Node::from_json_str("{\"a\":2}")?;
    /// let diff = lhs.diff(&rhs, &DiffOptions::default());
    /// let rendered = diff.render(&RenderConfig::default());
    /// assert_eq!(rendered, "@ [\"a\"]\n- 1\n+ 2\n");
    /// # Ok::<(), jd_core::CanonicalizeError>(())
    /// ```
    #[must_use]
    pub fn render(&self, config: &RenderConfig) -> String {
        let mut output = String::new();
        let mut inherited = DiffMetadata::default();
        for element in &self.elements {
            if let Some(metadata) = element.metadata.as_ref() {
                output.push_str(&metadata.render_header());
                inherited = metadata.clone();
            }
            let is_merge = element.metadata.as_ref().map_or(inherited.merge, |meta| meta.merge);
            output.push_str(&render_element_native(element, config, is_merge));
        }
        output
    }

    /// Renders the diff as a JSON Patch (RFC 6902).
    ///
    /// ```
    /// # use jd_core::{DiffOptions, Node};
    /// let lhs = Node::from_json_str("[1,2,3]")?;
    /// let rhs = Node::from_json_str("[1,4,3]")?;
    /// let diff = lhs.diff(&rhs, &DiffOptions::default());
    /// let patch = diff.render_patch()?;
    /// assert!(patch.starts_with("[{\"op\":\"test\""));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn render_patch(&self) -> Result<String, RenderError> {
        if self.is_empty() {
            return Ok("[]".to_string());
        }

        let mut operations = Vec::new();

        for element in &self.elements {
            if element.remove.is_empty() && element.add.is_empty() {
                return Err(RenderError::new("cannot render empty diff element as JSON Patch op"));
            }

            let pointer = path_to_pointer(&element.path)?;

            if element.before.len() > 1 {
                return Err(RenderError::new(format!(
                    "only one line of before context supported. got {}",
                    element.before.len()
                )));
            }
            if let Some(before) = element.before.first() {
                if !is_void(before) {
                    let last = element
                        .path
                        .segments()
                        .last()
                        .ok_or_else(|| RenderError::new("expected path. got empty path"))?;
                    let PathSegment::Index(index) = last else {
                        return Err(RenderError::new("wanted path index. got object key"));
                    };
                    let mut prev_path = element.path.clone();
                    prev_path.pop();
                    prev_path.push(PathSegment::Index(index - 1));
                    operations.push(PatchElement::test(
                        path_to_pointer(&prev_path)?,
                        node_to_json_value(before)?,
                    ));
                }
            }

            if element.after.len() > 1 {
                return Err(RenderError::new(format!(
                    "only one line of after context supported. got {}",
                    element.after.len()
                )));
            }
            if let Some(after) = element.after.first() {
                if !is_void(after) {
                    let last = element
                        .path
                        .segments()
                        .last()
                        .ok_or_else(|| RenderError::new("expected path. got empty path"))?;
                    let PathSegment::Index(index) = last else {
                        return Err(RenderError::new("wanted path index. got object key"));
                    };
                    let next_index = index + i64::try_from(element.remove.len()).unwrap_or(0);
                    let mut next_path = element.path.clone();
                    next_path.pop();
                    next_path.push(PathSegment::Index(next_index));
                    operations.push(PatchElement::test(
                        path_to_pointer(&next_path)?,
                        node_to_json_value(after)?,
                    ));
                }
            }

            if element.remove.first().map_or(false, |node| is_void(node)) {
                // Merge deletions encode void in remove; skip JSON Patch removal.
            } else {
                for value in &element.remove {
                    operations
                        .push(PatchElement::test(pointer.clone(), node_to_json_value(value)?));
                    operations
                        .push(PatchElement::remove(pointer.clone(), node_to_json_value(value)?));
                }
            }

            for value in element.add.iter().rev() {
                if is_void(value) {
                    continue;
                }
                operations.push(PatchElement::add(pointer.clone(), node_to_json_value(value)?));
            }
        }

        Ok(serde_json::to_string(&operations)?)
    }

    /// Renders the diff as a JSON Merge Patch (RFC 7386).
    ///
    /// ```
    /// # use jd_core::{diff::DiffElement, diff::PathSegment, Diff, DiffMetadata, Node};
    /// let element = DiffElement::new()
    ///     .with_metadata(DiffMetadata::merge())
    ///     .with_path(PathSegment::key("name"))
    ///     .with_add(vec![Node::from_json_str("\"jd\"").unwrap()]);
    /// let diff = Diff::from_elements(vec![element]);
    /// assert_eq!(diff.render_merge().unwrap(), "{\"name\":\"jd\"}");
    /// ```
    pub fn render_merge(&self) -> Result<String, RenderError> {
        if self.is_empty() {
            return Ok("{}".to_string());
        }

        let mut inherited = DiffMetadata::default();
        let mut normalized = Vec::with_capacity(self.elements.len());

        for element in &self.elements {
            if let Some(metadata) = element.metadata.as_ref() {
                inherited = metadata.clone();
            }
            let is_merge = element.metadata.as_ref().map_or(inherited.merge, |meta| meta.merge);
            if !is_merge {
                return Err(RenderError::new("cannot render non-merge element as merge"));
            }
            let mut clone = element.clone();
            for value in &mut clone.add {
                if is_void(value) {
                    *value = Node::Null;
                }
            }
            normalized.push(clone);
        }

        let diff = Diff::from_elements(normalized);
        let patched = Node::Void.apply_patch(&diff)?;
        let value = patched
            .to_json_value()
            .ok_or_else(|| RenderError::new("merge patch produced void value"))?;
        Ok(serde_json::to_string(&value)?)
    }

    /// Serializes the diff structure as JSON for debugging.
    pub fn render_raw(&self) -> Result<String, RenderError> {
        Ok(serde_json::to_string(&self.elements)?)
    }

    /// Reverses a strict diff so that applying it to the target restores the base value.
    ///
    /// ```
    /// # use jd_core::{DiffOptions, Node};
    /// let lhs = Node::from_json_str("{\"a\":1}")?;
    /// let rhs = Node::from_json_str("{\"a\":2}")?;
    /// let diff = lhs.diff(&rhs, &DiffOptions::default());
    /// let reversed = diff.reverse()?;
    /// let restored = rhs.apply_patch(&reversed)?;
    /// assert_eq!(restored, lhs);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn reverse(&self) -> Result<Diff, RenderError> {
        if self.elements.is_empty() {
            return Ok(Diff::default());
        }

        let mut active_metadata: Vec<Option<DiffMetadata>> =
            Vec::with_capacity(self.elements.len());
        let mut inherited: Option<DiffMetadata> = None;
        for element in &self.elements {
            if let Some(metadata) = element.metadata.as_ref().filter(|meta| meta.is_effective()) {
                if let Some(existing) = inherited.as_mut() {
                    existing.absorb(metadata);
                } else {
                    inherited = Some(metadata.clone());
                }
            }
            active_metadata.push(inherited.clone());
        }

        let mut reversed = Vec::with_capacity(self.elements.len());
        let mut last_emitted: Option<DiffMetadata> = None;

        for (index, element) in self.elements.iter().enumerate().rev() {
            let metadata = active_metadata[index].clone();
            if metadata.as_ref().map_or(false, |meta| meta.merge) {
                return Err(RenderError::new(format!(
                    "cannot reverse merge diff element at {}",
                    element.path
                )));
            }

            let mut clone = element.clone();
            std::mem::swap(&mut clone.remove, &mut clone.add);
            match metadata {
                Some(meta) => {
                    if last_emitted.as_ref() != Some(&meta) {
                        clone.metadata = Some(meta.clone());
                        last_emitted = Some(meta);
                    } else {
                        clone.metadata = None;
                    }
                }
                None => {
                    clone.metadata = None;
                    last_emitted = None;
                }
            }
            reversed.push(clone);
        }

        Ok(Diff::from_elements(reversed))
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

const COLOR_RESET: &str = "\u{1b}[0m";
const COLOR_RED: &str = "\u{1b}[31m";
const COLOR_GREEN: &str = "\u{1b}[32m";

#[derive(Serialize)]
struct PatchElement {
    op: &'static str,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<JsonValue>,
}

impl PatchElement {
    fn test(path: String, value: JsonValue) -> Self {
        Self { op: "test", path, value: Some(value) }
    }

    fn remove(path: String, value: JsonValue) -> Self {
        Self { op: "remove", path, value: Some(value) }
    }

    fn add(path: String, value: JsonValue) -> Self {
        Self { op: "add", path, value: Some(value) }
    }
}

fn is_void(node: &Node) -> bool {
    matches!(node, Node::Void)
}

fn render_element_native(element: &DiffElement, config: &RenderConfig, is_merge: bool) -> String {
    let mut output = String::new();
    output.push_str("@ ");
    output.push_str(&path_to_json(&element.path));
    output.push('\n');

    struct SingleStringDiff<'a> {
        common: Vec<char>,
        old: &'a str,
        new: &'a str,
    }

    let string_diff = if element.remove.len() == 1 && element.add.len() == 1 {
        match (&element.remove[0], &element.add[0]) {
            (Node::String(old), Node::String(new)) => {
                Some(SingleStringDiff { common: lcs_chars(old, new), old, new })
            }
            _ => None,
        }
    } else {
        None
    };

    for before in &element.before {
        if is_void(before) {
            output.push_str("[\n");
        } else {
            output.push_str("  ");
            output.push_str(&node_to_json(before));
            output.push('\n');
        }
    }

    for value in &element.remove {
        if is_void(value) {
            continue;
        }
        if let Some(diff) = &string_diff {
            if config.color_enabled() {
                output.push_str("- \"");
                output.push_str(&color_string_diff(diff.old, &diff.common, COLOR_RED));
                output.push_str("\"\n");
                continue;
            }
        }
        if config.color_enabled() {
            output.push_str(COLOR_RED);
        }
        output.push_str("- ");
        output.push_str(&node_to_json(value));
        output.push('\n');
        if config.color_enabled() {
            output.push_str(COLOR_RESET);
        }
    }

    for value in &element.add {
        if is_void(value) {
            if is_merge {
                if config.color_enabled() {
                    output.push_str(COLOR_GREEN);
                }
                output.push_str("+\n");
                if config.color_enabled() {
                    output.push_str(COLOR_RESET);
                }
            }
            continue;
        }
        if let Some(diff) = &string_diff {
            if config.color_enabled() {
                output.push_str("+ \"");
                output.push_str(&color_string_diff(diff.new, &diff.common, COLOR_GREEN));
                output.push_str("\"\n");
                continue;
            }
        }
        if config.color_enabled() {
            output.push_str(COLOR_GREEN);
        }
        output.push_str("+ ");
        output.push_str(&node_to_json(value));
        output.push('\n');
        if config.color_enabled() {
            output.push_str(COLOR_RESET);
        }
    }

    for after in &element.after {
        if is_void(after) {
            output.push_str("]\n");
        } else {
            output.push_str("  ");
            output.push_str(&node_to_json(after));
            output.push('\n');
        }
    }

    output
}

fn node_to_json(node: &Node) -> String {
    match node {
        Node::Void => String::new(),
        Node::Number(number) => json_number_from_f64(number.get()).to_string(),
        _ => {
            let value = node_to_json_value(node).expect("serializing node");
            serde_json::to_string(&value).expect("serializing node")
        }
    }
}

fn node_to_json_value(node: &Node) -> Result<JsonValue, RenderError> {
    match node {
        Node::Void => Err(RenderError::new("cannot encode void value in JSON Patch")),
        Node::Number(number) => Ok(JsonValue::Number(json_number_from_f64(number.get()))),
        _ => node
            .to_json_value()
            .ok_or_else(|| RenderError::new("cannot encode void value in JSON Patch")),
    }
}

fn path_to_json(path: &Path) -> String {
    let mut values = Vec::with_capacity(path.len());
    for segment in path.segments() {
        match segment {
            PathSegment::Key(key) => values.push(JsonValue::String(key.clone())),
            PathSegment::Index(index) => {
                let number =
                    JsonNumber::from_f64(*index as f64).expect("diff indices are finite values");
                values.push(JsonValue::Number(number));
            }
        }
    }
    serde_json::to_string(&JsonValue::Array(values)).expect("serialize path")
}

fn path_to_pointer(path: &Path) -> Result<String, RenderError> {
    let mut pointer = String::new();
    for segment in path.segments() {
        pointer.push('/');
        match segment {
            PathSegment::Index(index) => {
                if *index == -1 {
                    pointer.push('-');
                } else {
                    pointer.push_str(&index.to_string());
                }
            }
            PathSegment::Key(key) => {
                if key.parse::<i64>().is_ok() {
                    return Err(RenderError::new(format!(
                        "JSON Pointer does not support object keys that look like numbers: {}",
                        key
                    )));
                }
                if key == "-" {
                    return Err(RenderError::new("JSON Pointer does not support object key '-'"));
                }
                pointer.push_str(&escape_pointer_segment(key));
            }
        }
    }
    Ok(pointer)
}

fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn json_number_from_f64(value: f64) -> JsonNumber {
    if value.fract() == 0.0 {
        if (i64::MIN as f64) <= value && value <= (i64::MAX as f64) {
            return JsonNumber::from(value as i64);
        }
        if value >= 0.0 && value <= (u64::MAX as f64) {
            return JsonNumber::from(value as u64);
        }
    }
    JsonNumber::from_f64(value).expect("finite number")
}

fn color_string_diff(text: &str, common: &[char], color: &str) -> String {
    let mut result = String::new();
    let mut common_iter = common.iter();
    let mut current = common_iter.next();
    for ch in text.chars() {
        if let Some(expected) = current {
            if ch == *expected {
                result.push(ch);
                current = common_iter.next();
                continue;
            }
        }
        result.push_str(color);
        result.push(ch);
        result.push_str(COLOR_RESET);
    }
    result
}

fn lcs_chars(lhs: &str, rhs: &str) -> Vec<char> {
    let left: Vec<char> = lhs.chars().collect();
    let right: Vec<char> = rhs.chars().collect();
    let n = left.len();
    let m = right.len();
    let mut table = vec![vec![0usize; m + 1]; n + 1];
    for i in 0..n {
        for j in 0..m {
            if left[i] == right[j] {
                table[i + 1][j + 1] = table[i][j] + 1;
            } else {
                table[i + 1][j + 1] = table[i][j + 1].max(table[i + 1][j]);
            }
        }
    }

    let mut result = Vec::with_capacity(table[n][m]);
    let mut i = n;
    let mut j = m;
    while i > 0 && j > 0 {
        if left[i - 1] == right[j - 1] {
            result.push(left[i - 1]);
            i -= 1;
            j -= 1;
        } else if table[i - 1][j] >= table[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
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

    #[test]
    fn diff_of_arrays_with_nested_object_diff_preserves_child_path() {
        let lhs = Node::from_json_str("[{\"name\":\"jd\",\"version\":1}]").unwrap();
        let rhs = Node::from_json_str("[{\"name\":\"jd\",\"version\":2}]").unwrap();
        let diff = diff_nodes(&lhs, &rhs, &DiffOptions::default());
        let expected = Diff::from_elements(vec![DiffElement::new()
            .with_path(Path::from(vec![PathSegment::index(0), PathSegment::key("version")]))
            .with_remove(vec![Node::from_json_str("1").unwrap()])
            .with_add(vec![Node::from_json_str("2").unwrap()])]);
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
