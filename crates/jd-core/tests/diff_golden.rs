use std::fs;
use std::path::Path;

use jd_core::{Diff, DiffOptions, Node};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Fixture {
    lhs: String,
    rhs: String,
    diff: Diff,
}

fn load_fixture(path: &Path) -> Fixture {
    let data = fs::read_to_string(path).expect("fixture should be readable");
    serde_json::from_str(&data).expect("fixture should deserialize")
}

#[test]
fn list_mode_golden_parity() {
    let fixtures_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/diff/list");
    let mut entries: Vec<_> = fs::read_dir(&fixtures_root)
        .expect("fixtures directory must exist")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort();

    assert!(
        !entries.is_empty(),
        "expected at least one diff fixture under tests/fixtures/diff/list",
    );

    for path in entries {
        let fixture = load_fixture(&path);
        let lhs = Node::from_json_str(&fixture.lhs).expect("lhs parses");
        let rhs = Node::from_json_str(&fixture.rhs).expect("rhs parses");
        let diff = lhs.diff(&rhs, &DiffOptions::default());
        assert_eq!(diff, fixture.diff, "fixture {path:?}");
    }
}
