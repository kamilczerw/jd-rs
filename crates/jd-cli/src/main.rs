//! Command-line interface for the Rust port of the Go `jd` tool.
//!
//! This milestone wires the CLI to the renderer APIs implemented in
//! `jd-core`, supporting diff mode with native, JSON Patch, and JSON
//! Merge Patch outputs together with color toggling. Future milestones
//! will extend this binary with patch/translate modes and the remaining
//! flag surface.

use std::ffi::OsString;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use clap::{ArgAction, Parser, ValueEnum};
use jd_core::{DiffOptions, Node, RenderConfig};

const VERSION_BANNER: &str = concat!("jd version ", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    #[value(alias = "jd")]
    Native,
    #[value(alias = "patch")]
    Patch,
    #[value(alias = "merge")]
    Merge,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Native
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "jd",
    about = "Diff and patch JSON and YAML documents.",
    version = VERSION_BANNER,
    disable_help_subcommand = true,
    disable_version_flag = true,
    arg_required_else_help = false,
)]
struct Cli {
    /// Print version information and exit.
    #[arg(
        long = "version",
        action = ArgAction::SetTrue,
        help = "Print version information and exit.",
    )]
    version: bool,

    /// Render diff output using ANSI colors.
    #[arg(long = "color", action = ArgAction::SetTrue)]
    color: bool,

    /// Select diff output format (`jd`, `patch`, or `merge`).
    #[arg(short = 'f', long = "format", value_enum, default_value = "jd")]
    format: OutputFormat,

    /// Write output to FILE instead of STDOUT.
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// Enable patch mode (apply FILE1 patch to FILE2/STDIN).
    #[arg(short = 'p', action = ArgAction::SetTrue)]
    patch: bool,

    /// Translate mode (e.g. `jd2patch`).
    #[arg(short = 't', long = "translate")]
    translate: Option<String>,

    /// Read and write YAML instead of JSON.
    #[arg(long = "yaml", action = ArgAction::SetTrue)]
    yaml: bool,

    /// Numeric precision tolerance.
    #[arg(long = "precision")]
    precision: Option<f64>,

    /// Treat arrays as sets (not yet implemented).
    #[arg(long = "set", action = ArgAction::SetTrue)]
    set: bool,

    /// Treat arrays as multisets (not yet implemented).
    #[arg(long = "mset", action = ArgAction::SetTrue)]
    multiset: bool,

    /// Keys to identify objects within set semantics (not yet implemented).
    #[arg(long = "setkeys")]
    setkeys: Option<String>,

    /// Run as a git diff driver (not yet implemented).
    #[arg(long = "git-diff-driver", action = ArgAction::SetTrue)]
    git_diff_driver: bool,

    /// Serve the web UI on the provided port (not yet implemented).
    #[arg(long = "port")]
    port: Option<u16>,

    /// Positional inputs (FILE1 [FILE2]).
    #[arg()]
    inputs: Vec<OsString>,
}

fn main() {
    match try_main() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            let _ = writeln!(io::stderr(), "{err}");
            std::process::exit(1);
        }
    }
}

fn try_main() -> Result<i32> {
    let args = canonicalize_args(std::env::args_os());
    let cli = Cli::parse_from(args);

    if cli.version {
        println!("{VERSION_BANNER}");
        return Ok(0);
    }

    if cli.port.is_some() {
        bail!("The web UI (-port) is not supported in this build");
    }
    if cli.git_diff_driver {
        bail!("git diff driver mode is not implemented yet");
    }
    if cli.patch && cli.translate.is_some() {
        bail!("Patch and translate modes cannot be used together.");
    }

    let mode = if cli.patch {
        Mode::Patch
    } else if cli.translate.is_some() {
        Mode::Translate
    } else {
        Mode::Diff
    };

    match mode {
        Mode::Diff => run_diff(&cli),
        Mode::Patch => bail!("Patch mode is not implemented yet"),
        Mode::Translate => bail!("Translate mode is not implemented yet"),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Diff,
    Patch,
    Translate,
}

fn run_diff(cli: &Cli) -> Result<i32> {
    if cli.set {
        bail!("-set is not implemented yet");
    }
    if cli.multiset {
        bail!("-mset is not implemented yet");
    }
    if cli.setkeys.is_some() {
        bail!("-setkeys is not implemented yet");
    }

    let (first, second) = match cli.inputs.len() {
        1 => (InputSource::File(path_from(&cli.inputs[0])?), InputSource::Stdin),
        2 => (
            InputSource::File(path_from(&cli.inputs[0])?),
            InputSource::File(path_from(&cli.inputs[1])?),
        ),
        _ => {
            return Err(anyhow!(
                "Usage: jd [OPTION]... FILE1 [FILE2] -- expected one or two positional arguments"
            ))
        }
    };

    let lhs_text = read_input(&first)?;
    let rhs_text = read_input(&second)?;
    let lhs = parse_node(&lhs_text, cli.yaml).context("failed to parse first input")?;
    let rhs = parse_node(&rhs_text, cli.yaml).context("failed to parse second input")?;

    let options = build_options(cli)?;
    let diff = lhs.diff(&rhs, &options);

    let mut render_config = RenderConfig::default();
    if cli.color {
        render_config = render_config.with_color(true);
    }

    let (rendered, have_diff) = match cli.format {
        OutputFormat::Native => {
            let rendered = diff.render(&render_config);
            let have_diff = !rendered.is_empty();
            (rendered, have_diff)
        }
        OutputFormat::Patch => {
            let rendered = diff.render_patch().context("failed to render JSON Patch")?;
            let have_diff = rendered != "[]";
            (rendered, have_diff)
        }
        OutputFormat::Merge => {
            let rendered = diff.render_merge().context("failed to render merge patch")?;
            let have_diff = rendered != "{}";
            (rendered, have_diff)
        }
    };

    if let Some(path) = &cli.output {
        fs::write(path, rendered.as_bytes())
            .with_context(|| format!("failed to write output to {}", path.display()))?;
    } else {
        print!("{rendered}");
        io::stdout().flush().ok();
    }

    Ok(if have_diff { 1 } else { 0 })
}

#[derive(Debug)]
enum InputSource {
    File(PathBuf),
    Stdin,
}

fn path_from(input: &OsString) -> Result<PathBuf> {
    let path = PathBuf::from(input);
    if path.as_os_str().is_empty() {
        bail!("expected file path; got empty string");
    }
    Ok(path)
}

fn read_input(source: &InputSource) -> Result<String> {
    match source {
        InputSource::File(path) => {
            fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
        }
        InputSource::Stdin => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn parse_node(input: &str, yaml: bool) -> Result<Node> {
    if yaml {
        Node::from_yaml_str(input).map_err(|err| anyhow!(err))
    } else {
        Node::from_json_str(input).map_err(|err| anyhow!(err))
    }
}

fn build_options(cli: &Cli) -> Result<DiffOptions> {
    let mut options = DiffOptions::default();
    if let Some(precision) = cli.precision {
        options = options.with_precision(precision).map_err(|err| anyhow!(err))?;
    }
    Ok(options)
}

fn canonicalize_args<I>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = OsString>,
{
    let mut canonicalized = Vec::new();
    for (idx, arg) in args.into_iter().enumerate() {
        if idx == 0 {
            canonicalized.push(arg);
            continue;
        }
        match arg.to_str() {
            Some("-help") => canonicalized.push(OsString::from("--help")),
            Some("-version") => canonicalized.push(OsString::from("--version")),
            Some("-color") => canonicalized.push(OsString::from("--color")),
            Some(other) if other.starts_with("-f=") => {
                canonicalized.push(OsString::from("-f"));
                canonicalized.push(OsString::from(other.trim_start_matches("-f=")));
            }
            _ => canonicalized.push(arg),
        }
    }
    canonicalized
}

#[cfg(test)]
mod tests {
    use super::{canonicalize_args, OutputFormat};
    use std::ffi::OsString;

    #[test]
    fn canonicalizes_single_dash_variants() {
        let input = vec![
            OsString::from("jd"),
            OsString::from("-help"),
            OsString::from("-version"),
            OsString::from("--other"),
        ];
        let canonicalized = canonicalize_args(input.clone());
        assert_eq!(canonicalized[0], "jd");
        assert_eq!(canonicalized[1], "--help");
        assert_eq!(canonicalized[2], "--version");
        assert_eq!(canonicalized[3], "--other");
    }

    #[test]
    fn canonicalizes_inline_format_flag() {
        let input = vec![OsString::from("jd"), OsString::from("-f=patch")];
        let canonicalized = canonicalize_args(input);
        assert_eq!(canonicalized, vec!["jd", "-f", "patch"]);
    }

    #[test]
    fn output_format_default_is_native() {
        assert_eq!(OutputFormat::default(), OutputFormat::Native);
    }
}
