use jd_core::{DiffOptions, Node};

#[test]
fn jd_core_readme_example() -> Result<(), Box<dyn std::error::Error>> {
    let base = Node::from_json_str("[1,2,3]")?;
    let target = Node::from_json_str("[1,4,3]")?;

    let diff = base.diff(&target, &DiffOptions::default());
    assert_eq!(diff.len(), 1);

    let json_patch = diff.render_patch().expect("render patch");
    println!("{json_patch}");
    assert!(json_patch.contains("\"op\":\"test\""));

    let patched = base.apply_patch(&diff).expect("apply diff");
    assert_eq!(patched, target);
    Ok(())
}
