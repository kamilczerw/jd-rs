use assert_cmd::Command;
use predicates::prelude::*;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

#[derive(Debug, Deserialize)]
struct RenderOutputs {
    #[serde(default)]
    native: Option<String>,
    #[serde(default)]
    native_color: Option<String>,
    #[serde(default)]
    patch: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Fixture {
    lhs: String,
    rhs: String,
    render: RenderOutputs,
}

fn load_fixture(name: &str) -> Fixture {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../jd-core/tests/fixtures/render")
        .join(format!("{name}.json"));
    let data = fs::read_to_string(path).expect("fixture readable");
    serde_json::from_str(&data).expect("fixture deserializes")
}

fn write_tempfile(contents: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create tempfile");
    write!(file, "{contents}").expect("write tempfile");
    file
}

#[test]
fn help_succeeds() {
    let mut cmd = Command::cargo_bin("jd").expect("binary jd should be built");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Diff and patch JSON and YAML documents."));
}

#[test]
fn version_banner_matches_go_shape() {
    let mut cmd = Command::cargo_bin("jd").expect("binary jd should be built");
    cmd.arg("--version").assert().success().stdout(predicate::str::contains("jd version"));
}

#[test]
fn single_dash_version_is_normalized() {
    let mut cmd = Command::cargo_bin("jd").expect("binary jd should be built");
    cmd.arg("-version").assert().success().stdout(predicate::str::contains("jd version"));
}

#[test]
fn diff_native_matches_fixture() {
    let fixture = load_fixture("object_update");
    let expected = fixture.render.native.expect("native output available");
    let lhs = write_tempfile(&fixture.lhs);
    let rhs = write_tempfile(&fixture.rhs);

    let mut cmd = Command::cargo_bin("jd").expect("binary jd should be built");
    cmd.arg(lhs.path())
        .arg(rhs.path())
        .assert()
        .code(1)
        .stdout(expected)
        .stderr(predicate::str::is_empty());
}

#[test]
fn diff_patch_matches_fixture() {
    let fixture = load_fixture("object_update");
    let expected = fixture.render.patch.expect("patch output available");
    let lhs = write_tempfile(&fixture.lhs);
    let rhs = write_tempfile(&fixture.rhs);

    let mut cmd = Command::cargo_bin("jd").expect("binary jd should be built");
    cmd.arg("-f")
        .arg("patch")
        .arg(lhs.path())
        .arg(rhs.path())
        .assert()
        .code(1)
        .stdout(expected)
        .stderr(predicate::str::is_empty());
}

#[test]
fn diff_color_output_matches_fixture() {
    let fixture = load_fixture("string_diff_color");
    let expected = fixture.render.native_color.expect("color output available");
    let lhs = write_tempfile(&fixture.lhs);
    let rhs = write_tempfile(&fixture.rhs);

    let mut cmd = Command::cargo_bin("jd").expect("binary jd should be built");
    cmd.arg("--color")
        .arg(lhs.path())
        .arg(rhs.path())
        .assert()
        .code(1)
        .stdout(expected)
        .stderr(predicate::str::is_empty());
}

#[test]
fn diff_single_argument_reads_stdin() {
    let fixture = load_fixture("object_update");
    let expected = fixture.render.native.expect("native output available");
    let lhs = write_tempfile(&fixture.lhs);

    let mut cmd = Command::cargo_bin("jd").expect("binary jd should be built");
    cmd.arg(lhs.path())
        .write_stdin(fixture.rhs)
        .assert()
        .code(1)
        .stdout(expected)
        .stderr(predicate::str::is_empty());
}
