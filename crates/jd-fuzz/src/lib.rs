//! Fuzzing harnesses for the Rust port of the `jd` tool.
//!
//! The helpers in this crate are intentionally lightweight so they can be
//! reused both from `cargo fuzz` targets and from future property-based smoke
//! tests. Each public function accepts raw bytes and exercises different parts
//! of the parsing, diffing, and patching pipelines while swallowing any
//! recoverable errors.
//!
//! # Examples
//!
//! Run the canonicalization harness on a JSON snippet:
//!
//! ```
//! jd_fuzz::fuzz_canonicalization(b"{\"a\":1}");
//! ```
//!
//! Invoke the diff harness on deterministic input:
//!
//! ```
//! jd_fuzz::fuzz_diff(&[1, 2, 3, 4]);
//! ```
//!
//! Exercise the patch harness with arbitrary bytes:
//!
//! ```
//! jd_fuzz::fuzz_patch(b"example");
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use arbitrary::Unstructured;
use jd_core::{Diff, DiffOptions, Node};
use serde_json::{self, Map as JsonMap, Number as JsonNumber, Value as JsonValue};

const MAX_DEPTH: usize = 4;
const MAX_ARRAY_LEN: u8 = 6;
const MAX_OBJECT_LEN: u8 = 6;
const MAX_STRING_LEN: u8 = 12;

/// Feeds arbitrary bytes through the JSON and YAML canonicalization routines.
///
/// The function ignores decoding failures so that fuzzers can keep exploring.
///
/// ```
/// jd_fuzz::fuzz_canonicalization(b"{\"key\":\"value\"}");
/// ```
pub fn fuzz_canonicalization(data: &[u8]) {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = Node::from_json_str(text);
        let _ = Node::from_yaml_str(text);
    }
}

/// Drives the structural diff implementation with randomly generated nodes.
///
/// ```
/// jd_fuzz::fuzz_diff(b"seed");
/// ```
pub fn fuzz_diff(data: &[u8]) {
    let mut unstructured = Unstructured::new(data);
    let Some(lhs) = random_node(&mut unstructured) else {
        return;
    };
    let Some(rhs) = random_node(&mut unstructured) else {
        return;
    };
    let opts = DiffOptions::default();
    let diff = lhs.diff(&rhs, &opts);
    let _ = lhs.apply_patch(&diff);
    if let Ok(reversed) = diff.reverse() {
        let _ = rhs.apply_patch(&reversed);
    }
}

/// Applies both valid and arbitrary diffs to randomly generated nodes.
///
/// The harness first constructs a legitimate diff from two inputs and applies
/// it in both directions. It then attempts to deserialize an arbitrary diff
/// from the raw bytes and apply it to another random node to exercise error
/// paths.
///
/// ```
/// jd_fuzz::fuzz_patch(b"patch fuzz");
/// ```
pub fn fuzz_patch(data: &[u8]) {
    let mut unstructured = Unstructured::new(data);
    if let (Some(base), Some(target)) =
        (random_node(&mut unstructured), random_node(&mut unstructured))
    {
        let opts = DiffOptions::default();
        let diff = base.diff(&target, &opts);
        let _ = base.apply_patch(&diff);
        if let Ok(reversed) = diff.reverse() {
            let _ = target.apply_patch(&reversed);
        }
    }

    if let Ok(diff) = serde_json::from_slice::<Diff>(data) {
        let mut unstructured = Unstructured::new(data);
        if let Some(seed) = random_node(&mut unstructured) {
            let _ = seed.apply_patch(&diff);
        }
    }
}

fn random_node(unstructured: &mut Unstructured<'_>) -> Option<Node> {
    let value = json_value_from_unstructured(unstructured, 0).ok()?;
    Node::from_json_value(value).ok()
}

fn json_value_from_unstructured(
    unstructured: &mut Unstructured<'_>,
    depth: usize,
) -> Result<JsonValue, arbitrary::Error> {
    if depth >= MAX_DEPTH {
        return json_leaf(unstructured);
    }

    let choice = unstructured.int_in_range::<u8>(0..=5)?;
    match choice {
        0 => Ok(JsonValue::Null),
        1 => Ok(JsonValue::Bool(unstructured.arbitrary()?)),
        2 => Ok(JsonValue::Number(random_number(unstructured)?)),
        3 => Ok(JsonValue::String(random_string(unstructured)?)),
        4 => {
            let len = usize::from(unstructured.int_in_range::<u8>(0..=MAX_ARRAY_LEN)?);
            let mut items = Vec::with_capacity(len);
            for _ in 0..len {
                items.push(json_value_from_unstructured(unstructured, depth + 1)?);
            }
            Ok(JsonValue::Array(items))
        }
        _ => {
            let len = usize::from(unstructured.int_in_range::<u8>(0..=MAX_OBJECT_LEN)?);
            let mut map = JsonMap::new();
            for _ in 0..len {
                let key = random_string(unstructured)?;
                let value = json_value_from_unstructured(unstructured, depth + 1)?;
                map.insert(key, value);
            }
            Ok(JsonValue::Object(map))
        }
    }
}

fn json_leaf(unstructured: &mut Unstructured<'_>) -> Result<JsonValue, arbitrary::Error> {
    let choice = unstructured.int_in_range::<u8>(0..=3)?;
    match choice {
        0 => Ok(JsonValue::Null),
        1 => Ok(JsonValue::Bool(unstructured.arbitrary()?)),
        2 => Ok(JsonValue::Number(random_number(unstructured)?)),
        _ => Ok(JsonValue::String(random_string(unstructured)?)),
    }
}

fn random_number(unstructured: &mut Unstructured<'_>) -> Result<JsonNumber, arbitrary::Error> {
    if unstructured.arbitrary()? {
        let int = unstructured.arbitrary::<i64>()?;
        Ok(JsonNumber::from(int))
    } else {
        let numerator = unstructured.arbitrary::<i32>()? as f64;
        let denominator = f64::from(unstructured.int_in_range::<u16>(1..=1024)?);
        let value = numerator / denominator;
        JsonNumber::from_f64(value).ok_or(arbitrary::Error::IncorrectFormat)
    }
}

fn random_string(unstructured: &mut Unstructured<'_>) -> Result<String, arbitrary::Error> {
    let len = usize::from(unstructured.int_in_range::<u8>(0..=MAX_STRING_LEN)?);
    let mut string = String::with_capacity(len);
    for _ in 0..len {
        let byte = unstructured.int_in_range::<u8>(0x20..=0x7e)?;
        string.push(char::from(byte));
    }
    Ok(string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalization_handles_utf8() {
        fuzz_canonicalization(br"{}");
    }

    #[test]
    fn diff_harness_runs() {
        fuzz_diff(b"diff");
    }

    #[test]
    fn patch_harness_runs() {
        fuzz_patch(b"patch");
    }
}
