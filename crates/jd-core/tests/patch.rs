use jd_core::{diff::PathSegment, Diff, DiffElement, DiffMetadata, DiffOptions, Node};
use proptest::prop_assert_eq;

#[test]
fn apply_patch_replaces_scalar() {
    let base = Node::from_json_str("1").unwrap();
    let target = Node::from_json_str("2").unwrap();
    let diff = base.diff(&target, &DiffOptions::default());
    let patched = base.apply_patch(&diff).unwrap();
    assert_eq!(patched, target);
}

#[test]
fn apply_patch_handles_object_insertion() {
    let base = Node::from_json_str("{\"a\":1}").unwrap();
    let target = Node::from_json_str("{\"a\":1,\"b\":2}").unwrap();
    let diff = base.diff(&target, &DiffOptions::default());
    let patched = base.apply_patch(&diff).unwrap();
    assert_eq!(patched, target);
}

#[test]
fn apply_patch_list_context_validation_errors() {
    let base = Node::from_json_str("[1,2,3]").unwrap();
    let target = Node::from_json_str("[1,4,3]").unwrap();
    let diff = base.diff(&target, &DiffOptions::default());
    let mismatched = Node::from_json_str("[0,2,3]").unwrap();
    let err = mismatched.apply_patch(&diff).expect_err("patch should fail due to context mismatch");
    assert_eq!(err.to_string(), "invalid patch. expected 1 before. got 0");
}

#[test]
fn apply_patch_rejects_multiple_removals() {
    let element = DiffElement::new()
        .with_path(Vec::<PathSegment>::new())
        .with_remove(vec![Node::from_json_str("1").unwrap(), Node::from_json_str("2").unwrap()]);
    let diff = Diff::from_elements(vec![element]);
    let base = Node::from_json_str("1").unwrap();
    let err = base.apply_patch(&diff).expect_err("should reject multi-removal");
    assert_eq!(err.to_string(), "invalid diff: multiple removals from non-set at []");
}

#[test]
fn apply_patch_rejects_merge_old_value() {
    let element = DiffElement::new()
        .with_metadata(DiffMetadata::merge())
        .with_path(PathSegment::key("a"))
        .with_remove(vec![Node::from_json_str("1").unwrap()])
        .with_add(vec![Node::from_json_str("2").unwrap()]);
    let diff = Diff::from_elements(vec![element]);
    let base = Node::from_json_str("{\"a\":1}").unwrap();
    let err = base.apply_patch(&diff).expect_err("merge should reject old value");
    assert_eq!(err.to_string(), "patch with merge strategy at [a] has unnecessary old value 1");
}

fn arb_json_value() -> impl proptest::strategy::Strategy<Value = serde_json::Value> {
    use proptest::{collection::btree_map, collection::vec, prelude::*, string::string_regex};

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

    leaf.prop_recursive(3, 6, 4, move |inner| {
        prop_oneof![
            vec(inner.clone(), 0..4).prop_map(serde_json::Value::Array),
            btree_map(string_regex("[a-zA-Z0-9]{1,6}").unwrap(), inner, 0..4).prop_map(|map| {
                let mut object = serde_json::Map::new();
                for (k, v) in map {
                    object.insert(k, v);
                }
                serde_json::Value::Object(object)
            }),
        ]
    })
}

proptest::proptest! {
    #[test]
    fn diff_and_patch_roundtrip(a_json in arb_json_value(), b_json in arb_json_value()) {
        let a = Node::from_json_value(a_json.clone()).unwrap();
        let b = Node::from_json_value(b_json.clone()).unwrap();
        let opts = DiffOptions::default();
        let diff = a.diff(&b, &opts);
        let patched = a.apply_patch(&diff).unwrap();
        prop_assert_eq!(patched, b.clone());

        let reverse = b.diff(&a, &opts);
        let restored = b.apply_patch(&reverse).unwrap();
        prop_assert_eq!(restored, a);
    }

    #[test]
    fn empty_diff_is_idempotent(a_json in arb_json_value()) {
        let node = Node::from_json_value(a_json.clone()).unwrap();
        let diff = Diff::default();
        let expected = node.clone();
        let once = node.apply_patch(&diff).unwrap();
        prop_assert_eq!(once.clone(), expected.clone());
        let twice = once.apply_patch(&diff).unwrap();
        prop_assert_eq!(twice, expected);
    }
}
