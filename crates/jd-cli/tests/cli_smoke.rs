use assert_cmd::Command;
use predicates::prelude::*;

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
