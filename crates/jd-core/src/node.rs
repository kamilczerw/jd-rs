use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

use crate::{
    hash::{combine, hash_bytes, HashCode},
    ArrayMode, CanonicalizeError, DiffOptions, Number, PatchError,
};

const VOID_HASH: HashCode = [0xF3, 0x97, 0x6B, 0x21, 0x91, 0x26, 0x8D, 0x96];
const NULL_HASH: HashCode = [0xFE, 0x73, 0xAB, 0xCC, 0xE6, 0x32, 0xE0, 0x88];
const BOOL_TRUE_HASH: HashCode = [0x24, 0x6B, 0xE3, 0xE4, 0xAF, 0x59, 0xDC, 0x1C];
const BOOL_FALSE_HASH: HashCode = [0xC6, 0x38, 0x77, 0xD1, 0x0A, 0x7E, 0x1F, 0xBF];
const LIST_SEED: [u8; 8] = [0xF5, 0x18, 0x0A, 0x71, 0xA4, 0xC4, 0x03, 0xF3];
const OBJECT_SEED: [u8; 8] = [0x00, 0x5D, 0x39, 0xA4, 0x18, 0x10, 0xEA, 0xD5];

/// Represents the canonical JSON data model used by the diff engine.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Node {
    /// Sentinel representing the absence of a value.
    Void,
    /// JSON `null`.
    Null,
    /// JSON boolean.
    Bool(bool),
    /// JSON number represented as IEEE-754 double precision.
    Number(Number),
    /// JSON string.
    String(String),
    /// JSON array.
    Array(Vec<Node>),
    /// JSON object with deterministic key ordering.
    Object(BTreeMap<String, Node>),
}

impl Node {
    /// Parses a JSON string into the canonical node representation.
    ///
    /// ```
    /// # use jd_core::Node;
    /// let node = Node::from_json_str("{\"hello\":\"world\"}")?;
    /// assert!(matches!(node, Node::Object(_)));
    /// # Ok::<(), jd_core::CanonicalizeError>(())
    /// ```
    pub fn from_json_str(input: &str) -> Result<Self, CanonicalizeError> {
        if input.trim().is_empty() {
            return Ok(Self::Void);
        }
        let value: JsonValue = serde_json::from_str(input)?;
        Self::from_json_value(value)
    }

    /// Parses a YAML string into the canonical node representation.
    ///
    /// ```
    /// # use jd_core::Node;
    /// let node = Node::from_yaml_str("---\nanswer: 42\n")?;
    /// assert!(matches!(node, Node::Object(_)));
    /// # Ok::<(), jd_core::CanonicalizeError>(())
    /// ```
    pub fn from_yaml_str(input: &str) -> Result<Self, CanonicalizeError> {
        if input.trim().is_empty() {
            return Ok(Self::Void);
        }
        let value: YamlValue = serde_yaml::from_str(input)?;
        Self::from_yaml_value(value)
    }

    /// Converts a serde JSON value into a [`Node`].
    pub fn from_json_value(value: JsonValue) -> Result<Self, CanonicalizeError> {
        match value {
            JsonValue::Null => Ok(Self::Null),
            JsonValue::Bool(v) => Ok(Self::Bool(v)),
            JsonValue::Number(num) => {
                let text = num.to_string();
                let Some(as_f64) = num.as_f64() else {
                    return Err(CanonicalizeError::NumberOutOfRange { value: text });
                };
                Ok(Self::Number(Number::new(as_f64)?))
            }
            JsonValue::String(s) => Ok(Self::String(s)),
            JsonValue::Array(values) => {
                let mut items = Vec::with_capacity(values.len());
                for value in values {
                    items.push(Self::from_json_value(value)?);
                }
                Ok(Self::Array(items))
            }
            JsonValue::Object(map) => {
                let mut object = BTreeMap::new();
                for (key, value) in map {
                    object.insert(key, Self::from_json_value(value)?);
                }
                Ok(Self::Object(object))
            }
        }
    }

    fn from_yaml_value(value: YamlValue) -> Result<Self, CanonicalizeError> {
        match value {
            YamlValue::Null => Ok(Self::Null),
            YamlValue::Bool(v) => Ok(Self::Bool(v)),
            YamlValue::Number(num) => {
                if let Some(f) = num.as_f64() {
                    return Ok(Self::Number(Number::new(f)?));
                }
                if let Some(i) = num.as_i64() {
                    return Ok(Self::Number(Number::new(i as f64)?));
                }
                if let Some(u) = num.as_u64() {
                    return Ok(Self::Number(Number::new(u as f64)?));
                }
                Err(CanonicalizeError::NumberOutOfRange { value: num.to_string() })
            }
            YamlValue::String(s) => Ok(Self::String(s)),
            YamlValue::Sequence(seq) => {
                let mut items = Vec::with_capacity(seq.len());
                for value in seq {
                    items.push(Self::from_yaml_value(value)?);
                }
                Ok(Self::Array(items))
            }
            YamlValue::Mapping(map) => {
                let mut object = BTreeMap::new();
                for (key, value) in map {
                    let key = match key {
                        YamlValue::String(s) => s,
                        other => {
                            return Err(CanonicalizeError::NonStringYamlKey {
                                found: format!("{other:?}"),
                            });
                        }
                    };
                    object.insert(key, Self::from_yaml_value(value)?);
                }
                Ok(Self::Object(object))
            }
            YamlValue::Tagged(tagged) => {
                Err(CanonicalizeError::UnsupportedYamlTag { tag: tagged.tag.to_string() })
            }
        }
    }

    /// Converts the node into a serde JSON value when representable.
    ///
    /// Returns `None` when the node contains the `Void` sentinel (either at the
    /// root or nested within arrays/objects) because `serde_json::Value` cannot
    /// represent the absence of a value.
    #[must_use]
    pub fn to_json_value(&self) -> Option<JsonValue> {
        match self {
            Self::Void => None,
            Self::Null => Some(JsonValue::Null),
            Self::Bool(v) => Some(JsonValue::Bool(*v)),
            Self::Number(n) => Some(JsonValue::Number(n.to_json_number())),
            Self::String(s) => Some(JsonValue::String(s.clone())),
            Self::Array(values) => {
                let mut result = Vec::with_capacity(values.len());
                for value in values {
                    result.push(value.to_json_value()?);
                }
                Some(JsonValue::Array(result))
            }
            Self::Object(map) => {
                let mut object = serde_json::Map::new();
                for (key, value) in map {
                    object.insert(key.clone(), value.to_json_value()?);
                }
                Some(JsonValue::Object(object))
            }
        }
    }

    /// Structural equality that respects [`DiffOptions`].
    ///
    /// ```
    /// # use jd_core::{ArrayMode, DiffOptions, Node};
    /// let lhs = Node::from_json_str("[1,2]")?;
    /// let rhs = Node::from_json_str("[2,1]")?;
    /// let opts = DiffOptions::default().with_array_mode(ArrayMode::Set).expect("set mode");
    /// assert!(lhs.eq_with_options(&rhs, &opts));
    /// # Ok::<(), jd_core::CanonicalizeError>(())
    /// ```
    #[must_use]
    pub fn eq_with_options(&self, other: &Self, options: &DiffOptions) -> bool {
        match (self, other) {
            (Self::Void, Self::Void) => true,
            (Self::Null, Self::Null) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a.equals_with_precision(*b, options.precision()),
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Array(a), Self::Array(b)) => match options.array_mode() {
                ArrayMode::List => list_equals(a, b, options),
                ArrayMode::Set => set_equals(a, b, options),
                ArrayMode::MultiSet => multiset_equals(a, b, options),
            },
            (Self::Object(a), Self::Object(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (key, value_a) in a {
                    let Some(value_b) = b.get(key) else {
                        return false;
                    };
                    if !value_a.eq_with_options(value_b, options) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Computes the structural diff between two nodes.
    ///
    /// ```
    /// # use jd_core::{DiffOptions, Node};
    /// let lhs = Node::from_json_str("1").unwrap();
    /// let rhs = Node::from_json_str("2").unwrap();
    /// let diff = lhs.diff(&rhs, &DiffOptions::default());
    /// assert_eq!(diff.len(), 1);
    /// ```
    #[must_use]
    pub fn diff(&self, other: &Self, options: &DiffOptions) -> crate::Diff {
        crate::diff::diff_nodes(self, other, options)
    }

    /// Applies a diff to this node, returning the patched node on success.
    ///
    /// ```
    /// # use jd_core::{DiffOptions, Node};
    /// let base = Node::from_json_str("[1,2,3]")?;
    /// let target = Node::from_json_str("[1,4,3]")?;
    /// let diff = base.diff(&target, &DiffOptions::default());
    /// let patched = base.apply_patch(&diff)?;
    /// assert_eq!(patched, target);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn apply_patch(&self, diff: &crate::Diff) -> Result<Self, PatchError> {
        crate::patch::apply_patch(self, diff)
    }

    /// Computes the Go-compatible hash code for this node.
    #[must_use]
    pub fn hash_code(&self, options: &DiffOptions) -> HashCode {
        match self {
            Self::Void => VOID_HASH,
            Self::Null => NULL_HASH,
            Self::Bool(true) => BOOL_TRUE_HASH,
            Self::Bool(false) => BOOL_FALSE_HASH,
            Self::Number(n) => n.hash_code(),
            Self::String(s) => hash_bytes(s.as_bytes()),
            Self::Array(values) => match options.array_mode() {
                ArrayMode::List => hash_list(values, options),
                ArrayMode::Set => hash_set(values, options),
                ArrayMode::MultiSet => hash_multiset(values, options),
            },
            Self::Object(map) => hash_object(map, options),
        }
    }
}

impl TryFrom<JsonValue> for Node {
    type Error = CanonicalizeError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        Self::from_json_value(value)
    }
}

fn list_equals(lhs: &[Node], rhs: &[Node], options: &DiffOptions) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }
    lhs.iter().zip(rhs.iter()).all(|(a, b)| a.eq_with_options(b, options))
}

fn set_equals(lhs: &[Node], rhs: &[Node], options: &DiffOptions) -> bool {
    let lhs_hashes: BTreeSet<HashCode> = lhs.iter().map(|n| n.hash_code(options)).collect();
    let rhs_hashes: BTreeSet<HashCode> = rhs.iter().map(|n| n.hash_code(options)).collect();
    lhs_hashes == rhs_hashes
}

fn multiset_equals(lhs: &[Node], rhs: &[Node], options: &DiffOptions) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }
    let mut counts = BTreeMap::new();
    for hash in lhs.iter().map(|n| n.hash_code(options)) {
        *counts.entry(hash).or_insert(0usize) += 1;
    }
    for hash in rhs.iter().map(|n| n.hash_code(options)) {
        match counts.get_mut(&hash) {
            Some(count) if *count > 0 => *count -= 1,
            _ => return false,
        }
    }
    counts.values().all(|count| *count == 0)
}

fn hash_list(values: &[Node], options: &DiffOptions) -> HashCode {
    let mut bytes = Vec::with_capacity(8 + values.len() * 8);
    bytes.extend_from_slice(&LIST_SEED);
    for value in values {
        bytes.extend_from_slice(&value.hash_code(options));
    }
    hash_bytes(&bytes)
}

fn hash_set(values: &[Node], options: &DiffOptions) -> HashCode {
    let mut unique = BTreeSet::new();
    for value in values {
        unique.insert(value.hash_code(options));
    }
    combine(unique.into_iter().collect())
}

fn hash_multiset(values: &[Node], options: &DiffOptions) -> HashCode {
    let hashes: Vec<_> = values.iter().map(|n| n.hash_code(options)).collect();
    combine(hashes)
}

fn hash_object(map: &BTreeMap<String, Node>, options: &DiffOptions) -> HashCode {
    let mut bytes = Vec::with_capacity(OBJECT_SEED.len() + map.len() * 16);
    bytes.extend_from_slice(&OBJECT_SEED);
    for (key, value) in map {
        bytes.extend_from_slice(&hash_bytes(key.as_bytes()));
        bytes.extend_from_slice(&value.hash_code(options));
    }
    hash_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{
        collection::{btree_map, vec},
        prelude::*,
        string::string_regex,
    };

    fn arb_json_value() -> impl Strategy<Value = JsonValue> {
        let leaf = prop_oneof![
            Just(JsonValue::Null),
            any::<bool>().prop_map(JsonValue::Bool),
            proptest::num::f64::ANY.prop_filter_map("finite", |f| {
                if f.is_finite() {
                    serde_json::Number::from_f64(f).map(JsonValue::Number)
                } else {
                    None
                }
            }),
            string_regex("[a-zA-Z0-9]{0,8}").unwrap().prop_map(JsonValue::String),
        ];
        leaf.prop_recursive(4, 8, 4, move |inner| {
            prop_oneof![
                vec(inner.clone(), 0..4).prop_map(JsonValue::Array),
                btree_map(string_regex("[a-zA-Z0-9]{1,8}").unwrap(), inner, 0..4).prop_map(|map| {
                    let mut object = serde_json::Map::new();
                    for (k, v) in map {
                        object.insert(k, v);
                    }
                    JsonValue::Object(object)
                }),
            ]
        })
    }

    #[test]
    fn json_whitespace_is_void() {
        let node = Node::from_json_str("   \n\t").expect("whitespace should canonicalize to void");
        assert!(matches!(node, Node::Void));
    }

    #[test]
    fn json_object_roundtrip() {
        let node = Node::from_json_str("{\"a\":1,\"b\":true}").unwrap();
        let value = node.to_json_value().unwrap();
        assert_eq!(value["a"].as_f64().unwrap(), 1.0);
        assert!(value["b"].as_bool().unwrap());
    }

    #[test]
    fn json_number_to_json_value_is_minimal() {
        let node = Node::from_json_str("5").unwrap();
        let value = node.to_json_value().unwrap();
        assert_eq!(value, serde_json::json!(5));

        let neg_zero = Node::from_json_str("-0").unwrap();
        let neg_zero_value = neg_zero.to_json_value().unwrap();
        assert_eq!(serde_json::to_string(&neg_zero_value).unwrap(), "-0.0");
    }

    #[test]
    fn json_number_out_of_range_yields_error() {
        let err = Node::from_json_str("1e400").unwrap_err();
        match err {
            CanonicalizeError::NumberOutOfRange { .. } | CanonicalizeError::Json(_) => {}
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn yaml_non_string_key_errors() {
        let err = Node::from_yaml_str("? [1, 2]: 3").unwrap_err();
        let CanonicalizeError::NonStringYamlKey { .. } = err else {
            panic!("expected NonStringYamlKey error");
        };
    }

    #[test]
    fn number_precision_controls_equality() {
        let lhs = Node::from_json_str("1.0").unwrap();
        let rhs = Node::from_json_str("1.05").unwrap();
        let tight = DiffOptions::default();
        assert!(!lhs.eq_with_options(&rhs, &tight));
        let loose = DiffOptions::default().with_precision(0.1).unwrap();
        assert!(lhs.eq_with_options(&rhs, &loose));
    }

    #[test]
    fn array_mode_list_respects_order() {
        let lhs = Node::from_json_str("[1,2]").unwrap();
        let rhs = Node::from_json_str("[2,1]").unwrap();
        let opts = DiffOptions::default();
        assert!(!lhs.eq_with_options(&rhs, &opts));
    }

    #[test]
    fn array_mode_set_ignores_order() {
        let lhs = Node::from_json_str("[1,2]").unwrap();
        let rhs = Node::from_json_str("[2,1]").unwrap();
        let opts = DiffOptions::default().with_array_mode(ArrayMode::Set).unwrap();
        assert!(lhs.eq_with_options(&rhs, &opts));
    }

    proptest! {
        #[test]
        fn json_roundtrips_through_node(value in arb_json_value()) {
            let node = Node::from_json_value(value.clone()).unwrap();
            let reconstructed = node.to_json_value().unwrap();
            let node_again = Node::from_json_value(reconstructed.clone()).unwrap();
            prop_assert_eq!(node_again.clone(), node);
            let reconstructed_again = node_again.to_json_value().unwrap();
            prop_assert_eq!(reconstructed_again, reconstructed);
        }
    }
}
