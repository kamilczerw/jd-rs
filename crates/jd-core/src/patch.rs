//! Patch application engine for jd diffs.
//!
//! The implementation follows the semantics of the upstream Go version by
//! interpreting `DiffElement` metadata, enforcing list context validation, and
//! recursing through objects and arrays using strict or merge strategies.

use std::collections::BTreeMap;
use std::fmt;

use crate::{
    diff::{Path, PathSegment},
    Diff, DiffMetadata, Node,
};

/// Errors that can occur while applying a diff.
///
/// ```
/// # use jd_core::{DiffOptions, Node};
/// let base = Node::from_json_str("[1,2,3]").unwrap();
/// let target = Node::from_json_str("[1,4,3]").unwrap();
/// let diff = base.diff(&target, &DiffOptions::default());
/// let err = Node::from_json_str("[0,2,3]").unwrap().apply_patch(&diff).unwrap_err();
/// assert_eq!(err.to_string(), "invalid patch. expected 1 before. got 0");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchError {
    message: String,
}

impl PatchError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PatchError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PatchStrategy {
    Strict,
    Merge,
}

impl PatchStrategy {
    fn from_metadata(metadata: Option<&DiffMetadata>) -> Self {
        if metadata.is_some_and(|m| m.merge) {
            Self::Merge
        } else {
            Self::Strict
        }
    }
}

impl fmt::Display for PatchStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Strict => f.write_str("strict"),
            Self::Merge => f.write_str("merge"),
        }
    }
}

pub(crate) fn apply_patch(node: &Node, diff: &Diff) -> Result<Node, PatchError> {
    let mut current = node.clone();
    let mut inherited_metadata: Option<DiffMetadata> = None;
    for element in diff.iter() {
        if let Some(meta) = element.metadata.as_ref().filter(|metadata| metadata.is_effective()) {
            if let Some(existing) = inherited_metadata.as_mut() {
                existing.absorb(meta);
            } else {
                inherited_metadata = Some(meta.clone());
            }
        }
        let metadata = inherited_metadata.as_ref().filter(|metadata| metadata.is_effective());
        let strategy = PatchStrategy::from_metadata(metadata);
        current = patch_element(
            current,
            Vec::new(),
            element.path.segments(),
            &element.before,
            &element.remove,
            &element.add,
            &element.after,
            strategy,
        )?;
    }
    Ok(current)
}

// Mirrors the Go implementation signature for parity with the CLI contract.
#[allow(clippy::too_many_arguments)]
fn patch_element(
    node: Node,
    path_behind: Vec<PathSegment>,
    path_ahead: &[PathSegment],
    before: &[Node],
    remove: &[Node],
    add: &[Node],
    after: &[Node],
    strategy: PatchStrategy,
) -> Result<Node, PatchError> {
    if !path_ahead.is_empty() && strategy == PatchStrategy::Merge {
        let (segment, rest) = path_ahead.split_first().unwrap();
        let PathSegment::Key(key) = segment else {
            return Err(expected_collection_error(&node, segment));
        };

        match node {
            Node::Object(mut map) => {
                let existing = map.remove(key).unwrap_or_else(|| {
                    if rest.is_empty() {
                        Node::Void
                    } else {
                        Node::Object(BTreeMap::new())
                    }
                });
                let mut new_path = path_behind.clone();
                new_path.push(PathSegment::Key(key.clone()));
                let patched =
                    patch_element(existing, new_path, rest, before, remove, add, after, strategy)?;
                if is_void(&patched) && rest.is_empty() {
                    // Removal handled via map.remove above.
                } else if !is_void(&patched) || !rest.is_empty() {
                    map.insert(key.clone(), patched);
                }
                return Ok(Node::Object(map));
            }
            _other => {
                let seed = if rest.is_empty() { Node::Void } else { Node::Object(BTreeMap::new()) };
                let mut new_path = path_behind.clone();
                new_path.push(PathSegment::Key(key.clone()));
                let patched =
                    patch_element(seed, new_path, rest, before, remove, add, after, strategy)?;
                let mut map = BTreeMap::new();
                if !is_void(&patched) || !rest.is_empty() {
                    map.insert(key.clone(), patched);
                }
                return Ok(Node::Object(map));
            }
        }
    }

    match node {
        Node::Array(values) => {
            patch_list(values, path_behind, path_ahead, before, remove, add, after, strategy)
        }
        Node::Object(map) => {
            patch_object(map, path_behind, path_ahead, before, remove, add, after, strategy)
        }
        other => {
            if let Some(segment) = path_ahead.first() {
                return Err(expected_collection_error(&other, segment));
            }
            patch_scalar(other, path_behind, path_ahead, before, remove, add, after, strategy)
        }
    }
}

// Mirrors the Go implementation signature for parity with the CLI contract.
#[allow(clippy::too_many_arguments)]
fn patch_scalar(
    node: Node,
    path_behind: Vec<PathSegment>,
    path_ahead: &[PathSegment],
    _before: &[Node],
    old_values: &[Node],
    new_values: &[Node],
    _after: &[Node],
    strategy: PatchStrategy,
) -> Result<Node, PatchError> {
    if !path_ahead.is_empty() {
        if let Some(segment) = path_ahead.first() {
            return Err(expected_collection_error(&node, segment));
        }
    }
    if old_values.len() > 1 || new_values.len() > 1 {
        return Err(non_set_diff_error(old_values, new_values, &path_behind));
    }
    let old_value = single_value(old_values);
    let new_value = single_value(new_values);
    match strategy {
        PatchStrategy::Merge => {
            if !is_void(&old_value) {
                return Err(PatchError::new(format!(
                    "patch with merge strategy at {} has unnecessary old value {}",
                    path_to_string(&path_behind),
                    node_json(&old_value)
                )));
            }
        }
        PatchStrategy::Strict => {
            if !node_equals(&node, &old_value) {
                return Err(expect_value_error(&old_value, &node, &path_behind));
            }
        }
    }
    Ok(new_value)
}

// Mirrors the Go implementation signature for parity with the CLI contract.
#[allow(clippy::too_many_arguments)]
fn patch_object(
    mut map: BTreeMap<String, Node>,
    path_behind: Vec<PathSegment>,
    path_ahead: &[PathSegment],
    _before: &[Node],
    old_values: &[Node],
    new_values: &[Node],
    _after: &[Node],
    strategy: PatchStrategy,
) -> Result<Node, PatchError> {
    if path_ahead.is_empty() {
        if old_values.len() > 1 || new_values.len() > 1 {
            return Err(non_set_diff_error(old_values, new_values, &path_behind));
        }
        let new_value = single_value(new_values);
        if strategy == PatchStrategy::Merge {
            return Ok(new_value);
        }
        let old_value = single_value(old_values);
        if !node_equals(&Node::Object(map.clone()), &old_value) {
            return Err(expect_value_error(&old_value, &Node::Object(map), &path_behind));
        }
        return Ok(new_value);
    }

    let (segment, rest) = path_ahead.split_first().unwrap();
    let PathSegment::Key(key) = segment else {
        return Err(PatchError::new(format!(
            "found {} at {}: expected JSON object",
            node_json(&Node::Object(map.clone())),
            path_to_string(&path_behind)
        )));
    };

    let mut next = map.get(key).cloned();
    if next.is_none() {
        next = Some(match strategy {
            PatchStrategy::Merge => {
                if rest.is_empty() {
                    Node::Void
                } else {
                    Node::Object(BTreeMap::new())
                }
            }
            PatchStrategy::Strict => Node::Void,
        });
    }

    let mut new_path = path_behind.clone();
    new_path.push(PathSegment::Key(key.clone()));
    let patched =
        patch_element(next.unwrap(), new_path, rest, &[], old_values, new_values, &[], strategy)?;

    if is_void(&patched) {
        map.remove(key);
    } else {
        map.insert(key.clone(), patched);
    }
    Ok(Node::Object(map))
}

// Mirrors the Go implementation signature for parity with the CLI contract.
#[allow(clippy::too_many_arguments)]
fn patch_list(
    list: Vec<Node>,
    path_behind: Vec<PathSegment>,
    path_ahead: &[PathSegment],
    before: &[Node],
    remove: &[Node],
    add: &[Node],
    after: &[Node],
    strategy: PatchStrategy,
) -> Result<Node, PatchError> {
    if strategy == PatchStrategy::Merge {
        return patch_scalar(
            Node::Array(list),
            path_behind,
            path_ahead,
            before,
            remove,
            add,
            after,
            strategy,
        );
    }

    if path_ahead.is_empty() {
        if remove.len() > 1 || add.len() > 1 {
            return Err(PatchError::new("cannot replace list with multiple values"));
        }
        if remove.is_empty() {
            return Err(PatchError::new("invalid diff. must declare list to replace it"));
        }
        let wanted = &remove[0];
        let current = Node::Array(list);
        if !node_equals(&current, wanted) {
            return Err(PatchError::new(format!(
                "wanted {}. found {}",
                node_json(wanted),
                node_json(&current)
            )));
        }
        if add.is_empty() {
            return Ok(Node::Void);
        }
        return Ok(add[0].clone());
    }

    let (segment, rest) = path_ahead.split_first().unwrap();
    let PathSegment::Index(raw_index) = segment else {
        return Err(invalid_path_element_error(segment));
    };

    if !rest.is_empty() {
        if *raw_index < 0 || (*raw_index as usize) >= list.len() {
            return Err(PatchError::new(format!("patch index out of bounds: {raw_index}")));
        }
        let mut new_path = path_behind.clone();
        new_path.push(PathSegment::Index(*raw_index));
        let mut list_clone = list.clone();
        let child = list_clone[*raw_index as usize].clone();
        let patched = patch_element(child, new_path, rest, &[], remove, add, &[], strategy)?;
        list_clone[*raw_index as usize] = patched;
        return Ok(Node::Array(list_clone));
    }

    if *raw_index == -1 {
        if !remove.is_empty() {
            return Err(PatchError::new(
                "invalid patch. appending to -1 index. but want to remove values",
            ));
        }
        let mut list_clone = list.clone();
        list_clone.extend(add.iter().cloned());
        return Ok(Node::Array(list_clone));
    }

    if *raw_index < 0 {
        return Err(PatchError::new(format!("patch index out of bounds: {raw_index}")));
    }

    let insertion_index = *raw_index as usize;
    let original = list.clone();

    for (offset, context) in before.iter().enumerate() {
        let distance = before.len() - offset;
        let check_index = (*raw_index as isize) - (distance as isize);
        if check_index < 0 {
            if check_index == -1 && is_void(context) {
                continue;
            }
            return Err(PatchError::new(format!(
                "invalid patch. before context {} out of bounds: {check_index}",
                node_json(context)
            )));
        }
        let check_index = check_index as usize;
        if !node_equals(&original[check_index], context) {
            return Err(PatchError::new(format!(
                "invalid patch. expected {} before. got {}",
                node_json(context),
                node_json(&original[check_index])
            )));
        }
    }

    let mut working = original.clone();
    if !remove.is_empty() {
        if insertion_index >= working.len() {
            return Err(PatchError::new(format!("remove values out bounds: {raw_index}")));
        }
        for expected in remove {
            if !node_equals(&working[insertion_index], expected) {
                return Err(PatchError::new(format!(
                    "invalid patch. wanted {}. found {}",
                    node_json(expected),
                    node_json(&working[insertion_index])
                )));
            }
            working.remove(insertion_index);
        }
    }

    if insertion_index > working.len() {
        return Err(PatchError::new(format!("remove values out bounds: {raw_index}")));
    }

    let mut result = Vec::with_capacity(working.len() + add.len());
    result.extend(working.iter().take(insertion_index).cloned());
    result.extend(add.iter().cloned());
    result.extend(working.iter().skip(insertion_index).cloned());

    for (offset, context) in after.iter().enumerate() {
        let check_index = insertion_index + offset;
        if check_index >= working.len() {
            if check_index == working.len() && is_void(context) {
                continue;
            }
            return Err(PatchError::new(format!(
                "invalid patch. after context {} out of bounds: {check_index}",
                node_json(context)
            )));
        }
        if !node_equals(&working[check_index], context) {
            return Err(PatchError::new(format!(
                "invalid patch. expected {} after. got {}",
                node_json(context),
                node_json(&working[check_index])
            )));
        }
    }

    Ok(Node::Array(result))
}

fn non_set_diff_error(
    old_values: &[Node],
    _new_values: &[Node],
    path: &[PathSegment],
) -> PatchError {
    if old_values.len() > 1 {
        return PatchError::new(format!(
            "invalid diff: multiple removals from non-set at {}",
            path_to_string(path)
        ));
    }
    PatchError::new(format!(
        "invalid diff: multiple additions to a non-set at {}",
        path_to_string(path)
    ))
}

fn expect_value_error(expected: &Node, found: &Node, path: &[PathSegment]) -> PatchError {
    PatchError::new(format!(
        "found {} at {}: expected {}",
        node_json(found),
        path_to_string(path),
        node_json(expected)
    ))
}

fn expected_collection_error(node: &Node, segment: &PathSegment) -> PatchError {
    let expected = match segment {
        PathSegment::Key(_) => "JSON object",
        PathSegment::Index(_) => "JSON array",
    };
    PatchError::new(format!("found {} at {segment}: expected {expected}", node_json(node)))
}

fn invalid_path_element_error(segment: &PathSegment) -> PatchError {
    let type_name = match segment {
        PathSegment::Key(_) => "string",
        PathSegment::Index(_) => "float64",
    };
    PatchError::new(format!("invalid path element {type_name}: expected float64"))
}

fn single_value(values: &[Node]) -> Node {
    values.first().cloned().unwrap_or(Node::Void)
}

fn is_void(node: &Node) -> bool {
    matches!(node, Node::Void)
}

fn node_equals(lhs: &Node, rhs: &Node) -> bool {
    lhs == rhs
}

fn node_json(node: &Node) -> String {
    match node {
        Node::Void => String::new(),
        Node::Number(number) => {
            let value = number.get();
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                serde_json::Number::from_f64(value).map(|n| n.to_string()).unwrap_or_default()
            }
        }
        _ => match node.to_json_value() {
            Some(value) => serde_json::to_string(&value).unwrap_or_default(),
            None => String::new(),
        },
    }
}

fn path_to_string(path: &[PathSegment]) -> String {
    Path::from(path.to_vec()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_json_void() {
        assert_eq!(node_json(&Node::Void), "");
    }

    #[test]
    fn node_json_number_is_minimal() {
        let node = Node::from_json_str("1").unwrap();
        let rendered = node_json(&node);
        assert_eq!(rendered, "1");

        let json_number = serde_json::Number::from_f64(1.0).unwrap();
        assert_eq!(json_number.to_string(), "1.0");
    }
}
