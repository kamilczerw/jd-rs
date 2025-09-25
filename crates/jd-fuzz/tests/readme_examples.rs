#[test]
fn jd_fuzz_readme_example() {
    jd_fuzz::fuzz_diff(b"example");
}
