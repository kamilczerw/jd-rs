use std::fs;
use std::path::Path;

use jd_core::{Diff, DiffOptions, Node, RenderConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RenderOutputs {
    #[serde(default)]
    native: Option<String>,
    #[serde(default)]
    native_color: Option<String>,
    #[serde(default)]
    patch: Option<String>,
    #[serde(default)]
    merge: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Fixture {
    lhs: String,
    rhs: String,
    #[serde(default)]
    options: Vec<String>,
    diff: Diff,
    render: RenderOutputs,
}

fn load_fixture(path: &Path) -> Fixture {
    let data = fs::read_to_string(path).expect("fixture should be readable");
    serde_json::from_str(&data).expect("fixture should deserialize")
}

#[test]
fn render_parity_matches_go_outputs() {
    let fixtures_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/render");
    let mut entries: Vec<_> = fs::read_dir(&fixtures_root)
        .expect("fixtures directory must exist")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort();

    assert!(
        !entries.is_empty(),
        "expected at least one render fixture under tests/fixtures/render",
    );

    for path in entries {
        let fixture = load_fixture(&path);
        let lhs = Node::from_json_str(&fixture.lhs).expect("lhs parses");
        let rhs = Node::from_json_str(&fixture.rhs).expect("rhs parses");

        let diff = if fixture.options.iter().any(|opt| opt == "merge") {
            fixture.diff
        } else {
            let computed = lhs.diff(&rhs, &DiffOptions::default());
            assert_eq!(computed, fixture.diff, "fixture {path:?} diff");
            computed
        };

        if let Some(expected) = fixture.render.native {
            let rendered = diff.render(&RenderConfig::default());
            assert_eq!(rendered, expected, "fixture {path:?} native output");
        }

        if let Some(expected) = fixture.render.native_color {
            let rendered = diff.render(&RenderConfig::default().with_color(true));
            assert_eq!(rendered, expected, "fixture {path:?} native color output");
        }

        if let Some(expected) = fixture.render.patch {
            let rendered = diff.render_patch().expect("render_patch");
            assert_eq!(rendered, expected, "fixture {path:?} patch output");
        }

        if let Some(expected) = fixture.render.merge {
            let rendered = diff.render_merge().expect("render_merge");
            assert_eq!(rendered, expected, "fixture {path:?} merge output");
        }
    }
}
