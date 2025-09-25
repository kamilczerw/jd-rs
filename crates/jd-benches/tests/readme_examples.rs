use jd_benches::available_corpora;
use jd_core::{DiffOptions, RenderConfig};

#[test]
fn jd_benches_readme_example() -> Result<(), Box<dyn std::error::Error>> {
    let corpus =
        available_corpora().iter().find(|c| c.name() == "github-issue").expect("registered corpus");
    let dataset = corpus.load()?;
    let diff = dataset.diff(&DiffOptions::default());
    assert!(!diff.is_empty());

    let rendered = dataset.render_native(&diff, &RenderConfig::default());
    println!("{rendered}");
    assert!(rendered.contains("@ "));
    Ok(())
}
