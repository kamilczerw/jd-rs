use jd_core::{
    diff::PathSegment, Diff, DiffElement, DiffMetadata, DiffOptions, Node, RenderConfig,
};
use proptest::prelude::*;

fn simple_diff() -> Diff {
    let lhs = Node::from_json_str("{\"a\":1}").unwrap();
    let rhs = Node::from_json_str("{\"a\":2}").unwrap();
    lhs.diff(&rhs, &DiffOptions::default())
}

#[test]
fn render_native_object_replacement() {
    let diff = simple_diff();
    let rendered = diff.render(&RenderConfig::default());
    assert_eq!(rendered, "@ [\"a\"]\n- 1\n+ 2\n");
}

#[test]
fn render_native_string_diff_colorizes() {
    let lhs = Node::from_json_str("\"kitten\"").unwrap();
    let rhs = Node::from_json_str("\"sitting\"").unwrap();
    let diff = lhs.diff(&rhs, &DiffOptions::default());
    let rendered = diff.render(&RenderConfig::default().with_color(true));
    assert!(rendered.contains("\u{1b}[31m"), "expected ANSI red segment");
    assert!(rendered.contains("\u{1b}[32m"), "expected ANSI green segment");
}

#[test]
fn render_patch_emits_context_tests() {
    let lhs = Node::from_json_str("[1,2,3]").unwrap();
    let rhs = Node::from_json_str("[1,4,3]").unwrap();
    let diff = lhs.diff(&rhs, &DiffOptions::default());
    let patch = diff.render_patch().expect("render_patch");
    assert_eq!(
        patch,
        "[{\"op\":\"test\",\"path\":\"/0\",\"value\":1},{\"op\":\"test\",\"path\":\"/2\",\"value\":3},{\"op\":\"test\",\"path\":\"/1\",\"value\":2},{\"op\":\"remove\",\"path\":\"/1\",\"value\":2},{\"op\":\"add\",\"path\":\"/1\",\"value\":4}]"
    );
}

#[test]
fn render_patch_rejects_extra_context() {
    let element = DiffElement::new()
        .with_path(PathSegment::index(0))
        .with_before(vec![Node::Null, Node::Null])
        .with_remove(vec![Node::Null]);
    let diff = Diff::from_elements(vec![element]);
    let err = diff.render_patch().unwrap_err();
    assert_eq!(err.to_string(), "only one line of before context supported. got 2");
}

#[test]
fn render_patch_rejects_numeric_object_keys() {
    let element = DiffElement::new()
        .with_path(PathSegment::key("0"))
        .with_remove(vec![Node::Null])
        .with_add(vec![Node::Null]);
    let diff = Diff::from_elements(vec![element]);
    let err = diff.render_patch().unwrap_err();
    assert!(err
        .to_string()
        .contains("JSON Pointer does not support object keys that look like numbers"));
}

#[test]
fn render_merge_outputs_object() {
    let element = DiffElement::new()
        .with_metadata(DiffMetadata::merge())
        .with_path(PathSegment::key("name"))
        .with_add(vec![Node::from_json_str("\"jd\"").unwrap()]);
    let diff = Diff::from_elements(vec![element]);
    let rendered = diff.render_merge().expect("render_merge");
    assert_eq!(rendered, "{\"name\":\"jd\"}");
}

#[test]
fn render_merge_requires_merge_metadata() {
    let element = DiffElement::new()
        .with_path(PathSegment::key("name"))
        .with_add(vec![Node::from_json_str("\"jd\"").unwrap()]);
    let diff = Diff::from_elements(vec![element]);
    let err = diff.render_merge().unwrap_err();
    assert_eq!(err.to_string(), "cannot render non-merge element as merge");
}

#[test]
fn render_raw_serializes_diff() {
    let diff = simple_diff();
    let raw = diff.render_raw().expect("render_raw");
    let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 1);
}

#[test]
fn reverse_swaps_add_remove() {
    let diff = simple_diff();
    let reversed = diff.reverse().expect("reverse");
    let base = Node::from_json_str("{\"a\":2}").unwrap();
    let patched = base.apply_patch(&reversed).expect("patch");
    assert_eq!(patched, Node::from_json_str("{\"a\":1}").unwrap());
}

#[test]
fn reverse_rejects_merge_diffs() {
    let element = DiffElement::new()
        .with_metadata(DiffMetadata::merge())
        .with_path(PathSegment::key("a"))
        .with_add(vec![Node::from_json_str("1").unwrap()]);
    let diff = Diff::from_elements(vec![element]);
    let err = diff.reverse().unwrap_err();
    assert_eq!(err.to_string(), "cannot reverse merge diff element at [a]");
}

fn arb_json_value() -> impl Strategy<Value = serde_json::Value> {
    use proptest::{collection, string::string_regex};

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
        string_regex("[a-zA-Z0-9]{0,6}").unwrap().prop_map(serde_json::Value::String),
    ];

    leaf.prop_recursive(3, 6, 4, |inner| {
        prop_oneof![
            collection::vec(inner.clone(), 0..4).prop_map(serde_json::Value::Array),
            collection::btree_map(string_regex("[a-zA-Z0-9]{1,6}").unwrap(), inner, 0..4).prop_map(
                |map| {
                    let mut object = serde_json::Map::new();
                    for (k, v) in map {
                        object.insert(k, v);
                    }
                    serde_json::Value::Object(object)
                }
            ),
        ]
    })
}

proptest! {
    #[test]
    fn reverse_round_trip_property(a_json in arb_json_value(), b_json in arb_json_value()) {
        let a = Node::from_json_value(a_json.clone()).unwrap();
        let b = Node::from_json_value(b_json.clone()).unwrap();
        let opts = DiffOptions::default();
        let diff = a.diff(&b, &opts);
        let reversed = diff.reverse().unwrap();
        let forward = a.apply_patch(&diff).unwrap();
        prop_assert_eq!(forward, b.clone());
        let backward = b.apply_patch(&reversed).unwrap();
        prop_assert_eq!(backward, a);
    }

    #[test]
    fn reverse_is_involution(a_json in arb_json_value(), b_json in arb_json_value()) {
        let a = Node::from_json_value(a_json.clone()).unwrap();
        let b = Node::from_json_value(b_json.clone()).unwrap();
        let diff = a.diff(&b, &DiffOptions::default());
        let reversed = diff.reverse().unwrap();
        let round_trip = reversed.reverse().unwrap();
        prop_assert_eq!(round_trip, diff);
    }
}
