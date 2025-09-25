use jd_core::{DiffOptions, Node};

#[test]
fn top_level_readme_quickstart() -> Result<(), Box<dyn std::error::Error>> {
    let base = Node::from_json_str("{\"count\":1}")?;
    let target = Node::from_json_str("{\"count\":2}")?;

    let diff = base.diff(&target, &DiffOptions::default());
    assert!(!diff.is_empty());

    let patched = base.apply_patch(&diff)?;
    assert_eq!(patched, target);
    Ok(())
}
