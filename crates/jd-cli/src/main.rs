//! Command-line interface for the Rust port of the Go `jd` tool.
//!
//! This milestone wires the CLI to the renderer APIs implemented in
//! `jd-core`, supporting diff mode with native, JSON Patch, and JSON
//! Merge Patch outputs together with color toggling. Future milestones
//! will extend this binary with patch/translate modes and the remaining
//! flag surface.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use clap::{ArgAction, Parser, ValueEnum};
use jd_core::{ArrayMode, DiffOptions, Node, RenderConfig};
use serde_json::Value;

const VERSION_NUMBER: &str = env!("CARGO_PKG_VERSION");
const VERSION_BANNER: &str = concat!("jd version ", env!("CARGO_PKG_VERSION"));

const HELP_TEMPLATE: &str = r#"Usage: jd [OPTION]... FILE1 [FILE2]
Diff and patch JSON files.

Prints the diff of FILE1 and FILE2 to STDOUT.
When FILE2 is omitted the second input is read from STDIN.
When patching (-p) FILE1 is a diff.

Options:
  -color       Print color diff.
  -p           Apply patch FILE1 to FILE2 or STDIN.
  -o=FILE3     Write to FILE3 instead of STDOUT.
  -opts='[]'   JSON array of options ("SET", "MULTISET", {"precision":N}, {"setkeys":[...]}).
  -set         Treat arrays as sets.
  -mset        Treat arrays as multisets (bags).
  -setkeys     Keys to identify set objects
  -yaml        Read and write YAML instead of JSON.
  -port=N      Serve web UI on port N
  -precision=N Maximum absolute difference for numbers to be equal.
               Example: -precision=0.00001
  -f=FORMAT    Read and write diff in FORMAT "jd" (default), "patch" (RFC 6902) or
               "merge" (RFC 7386)
  -t=FORMATS   Translate FILE1 between FORMATS. Supported formats are "jd",
               "patch" (RFC 6902), "merge" (RFC 7386), "json" and "yaml".
               FORMATS are provided as a pair separated by "2". E.g.
               "yaml2json" or "jd2patch".

Examples:
  jd a.json b.json
  cat b.json | jd a.json
  jd -o patch a.json b.json; jd patch a.json
  jd -set a.json b.json
  jd -f patch a.json b.json
  jd -f merge a.json b.json

Version: {version}
"#;

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
    disable_help_flag = true,
    disable_help_subcommand = true,
    disable_version_flag = true,
    override_usage = "jd [OPTION]... FILE1 [FILE2]"
)]
struct Cli {
    #[arg(long = "help", short = 'h', action = ArgAction::SetTrue, hide = true)]
    help: bool,

    #[arg(long = "version", action = ArgAction::SetTrue, hide = true)]
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

    /// JSON-encoded diff options (mirrors Go `-opts`).
    #[arg(long = "opts", default_value = "[]")]
    opts: String,

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

    #[arg(long = "v2", action = ArgAction::SetTrue, hide = true)]
    v2: bool,

    /// Positional inputs (FILE1 \[FILE2]).
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

    if cli.help {
        print!("{}", help_text());
        return Ok(0);
    }

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
    let (first, second) = match cli.inputs.len() {
        1 => (InputSource::File(path_from(&cli.inputs[0])?), InputSource::Stdin),
        2 => (
            InputSource::File(path_from(&cli.inputs[0])?),
            InputSource::File(path_from(&cli.inputs[1])?),
        ),
        _ => {
            return Err(anyhow!("{}", help_text()));
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
            let patch = merge_patch(&lhs, &rhs).unwrap_or_else(|| Node::Object(BTreeMap::new()));
            let rendered = patch
                .to_json_value()
                .map(|value| serde_json::to_string(&value))
                .transpose()
                .context("failed to serialize merge patch")?
                .unwrap_or_else(|| "{}".to_string());
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

    for option in parse_opts_json(&cli.opts)? {
        options = apply_parsed_option(options, option)?;
    }

    if cli.set && cli.multiset {
        bail!("-set and -mset cannot be combined");
    }

    if cli.set {
        options = options.with_array_mode(ArrayMode::Set).map_err(|err| anyhow!(err))?;
    }

    if cli.multiset {
        options = options.with_array_mode(ArrayMode::MultiSet).map_err(|err| anyhow!(err))?;
    }

    if let Some(setkeys) = &cli.setkeys {
        let keys = parse_flag_set_keys(setkeys)?;
        options = options.with_set_keys(keys).map_err(|err| anyhow!(err))?;
    }

    if let Some(precision) = cli.precision {
        options = options.with_precision(precision).map_err(|err| anyhow!(err))?;
    }

    Ok(options)
}

#[derive(Debug)]
enum ParsedOption {
    ArrayMode(ArrayMode),
    Precision(f64),
    SetKeys(Vec<String>),
}

fn apply_parsed_option(options: DiffOptions, option: ParsedOption) -> Result<DiffOptions> {
    match option {
        ParsedOption::ArrayMode(mode) => options.with_array_mode(mode).map_err(|err| anyhow!(err)),
        ParsedOption::Precision(value) => options.with_precision(value).map_err(|err| anyhow!(err)),
        ParsedOption::SetKeys(keys) => options.with_set_keys(keys).map_err(|err| anyhow!(err)),
    }
}

fn parse_flag_set_keys(raw: &str) -> Result<Vec<String>> {
    let mut keys = Vec::new();
    for segment in raw.split(',') {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            bail!("invalid set key: {segment}");
        }
        keys.push(trimmed.to_string());
    }
    if keys.is_empty() {
        bail!("-setkeys requires at least one key");
    }
    Ok(keys)
}

fn parse_opts_json(raw: &str) -> Result<Vec<ParsedOption>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("-opts requires a JSON array"));
    }

    let value: Value = serde_json::from_str(trimmed)
        .with_context(|| format!("failed to parse -opts JSON: {trimmed}"))?;
    let items = match value {
        Value::Array(items) => items,
        other => return Err(anyhow!("-opts expects a JSON array, but received {}", other)),
    };

    let mut parsed = Vec::new();
    for item in &items {
        match item {
            Value::String(name) => match name.as_str() {
                "SET" => parsed.push(ParsedOption::ArrayMode(ArrayMode::Set)),
                "MULTISET" => parsed.push(ParsedOption::ArrayMode(ArrayMode::MultiSet)),
                other => {
                    bail!("unsupported -opts option: {other}");
                }
            },
            Value::Object(map) => {
                if map.contains_key("@") || map.contains_key("^") {
                    bail!("path-specific -opts entries are not supported yet");
                }
                if map.len() != 1 {
                    bail!("unsupported -opts object: {item}");
                }
                let (key, value) = map.iter().next().unwrap();
                match key.as_str() {
                    "precision" => {
                        let number = value
                            .as_f64()
                            .ok_or_else(|| anyhow!("precision option expects a numeric value"))?;
                        parsed.push(ParsedOption::Precision(number));
                    }
                    "setkeys" => {
                        let arr = value
                            .as_array()
                            .ok_or_else(|| anyhow!("setkeys option expects an array of strings"))?;
                        let mut keys = Vec::new();
                        for key_value in arr {
                            let key_str = key_value.as_str().ok_or_else(|| {
                                anyhow!("setkeys option expects an array of strings")
                            })?;
                            keys.push(key_str.to_string());
                        }
                        if keys.is_empty() {
                            bail!("setkeys option requires at least one key");
                        }
                        parsed.push(ParsedOption::SetKeys(keys));
                    }
                    other => {
                        bail!("unsupported -opts object option: {other}");
                    }
                }
            }
            _ => {
                bail!("unsupported -opts entry: {item}");
            }
        }
    }

    Ok(parsed)
}

fn merge_patch(lhs: &Node, rhs: &Node) -> Option<Node> {
    match (lhs, rhs) {
        (Node::Object(a), Node::Object(b)) => {
            let mut keys: BTreeSet<&String> = BTreeSet::new();
            keys.extend(a.keys());
            keys.extend(b.keys());

            let mut map = BTreeMap::new();
            for key in keys {
                match (a.get(key), b.get(key)) {
                    (Some(left), Some(right)) => {
                        if let Some(child) = merge_patch(left, right) {
                            match &child {
                                Node::Object(children) if children.is_empty() => {}
                                _ => {
                                    map.insert(key.clone(), child);
                                }
                            }
                        }
                    }
                    (Some(_), None) => {
                        map.insert(key.clone(), Node::Null);
                    }
                    (None, Some(value)) => {
                        map.insert(key.clone(), value.clone());
                    }
                    (None, None) => {}
                }
            }

            if map.is_empty() {
                None
            } else {
                Some(Node::Object(map))
            }
        }
        _ => {
            if lhs == rhs {
                None
            } else {
                Some(rhs.clone())
            }
        }
    }
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
            Some("-h") => canonicalized.push(OsString::from("--help")),
            Some("-version") => canonicalized.push(OsString::from("--version")),
            Some("-color") => canonicalized.push(OsString::from("--color")),
            Some("-yaml") => canonicalized.push(OsString::from("--yaml")),
            Some("-set") => canonicalized.push(OsString::from("--set")),
            Some("-mset") => canonicalized.push(OsString::from("--mset")),
            Some("-precision") => canonicalized.push(OsString::from("--precision")),
            Some("-setkeys") => canonicalized.push(OsString::from("--setkeys")),
            Some("-opts") => canonicalized.push(OsString::from("--opts")),
            Some("-v2") => canonicalized.push(OsString::from("--v2")),
            Some(other) if other.starts_with("-f=") => {
                canonicalized.push(OsString::from("-f"));
                canonicalized.push(OsString::from(other.trim_start_matches("-f=")));
            }
            Some(other) if other.starts_with("-precision=") => {
                canonicalized.push(OsString::from("--precision"));
                canonicalized.push(OsString::from(other.trim_start_matches("-precision=")));
            }
            Some(other) if other.starts_with("-setkeys=") => {
                canonicalized.push(OsString::from("--setkeys"));
                canonicalized.push(OsString::from(other.trim_start_matches("-setkeys=")));
            }
            Some(other) if other.starts_with("-opts=") => {
                canonicalized.push(OsString::from("--opts"));
                canonicalized.push(OsString::from(other.trim_start_matches("-opts=")));
            }
            _ => canonicalized.push(arg),
        }
    }
    canonicalized
}

fn help_text() -> String {
    HELP_TEMPLATE.replace("{version}", VERSION_NUMBER)
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
            OsString::from("-h"),
            OsString::from("-version"),
            OsString::from("-v2"),
            OsString::from("--other"),
        ];
        let canonicalized = canonicalize_args(input.clone());
        assert_eq!(canonicalized[0], "jd");
        assert_eq!(canonicalized[1], "--help");
        assert_eq!(canonicalized[2], "--help");
        assert_eq!(canonicalized[3], "--version");
        assert_eq!(canonicalized[4], "--v2");
        assert_eq!(canonicalized[5], "--other");
    }

    #[test]
    fn canonicalizes_inline_format_flag() {
        let input = vec![OsString::from("jd"), OsString::from("-f=patch")];
        let canonicalized = canonicalize_args(input);
        assert_eq!(canonicalized, vec!["jd", "-f", "patch"]);
    }

    #[test]
    fn canonicalizes_single_dash_long_flags() {
        let input = vec![
            OsString::from("jd"),
            OsString::from("-yaml"),
            OsString::from("-precision"),
            OsString::from("0.01"),
            OsString::from("-precision=0.02"),
            OsString::from("-set"),
            OsString::from("-mset"),
            OsString::from("-setkeys"),
            OsString::from("id"),
            OsString::from("-setkeys=name"),
            OsString::from("-opts"),
            OsString::from("[\"SET\"]"),
            OsString::from("-opts=[{\"precision\":0.1}]"),
        ];
        let canonicalized = canonicalize_args(input);
        assert_eq!(
            canonicalized,
            vec![
                OsString::from("jd"),
                OsString::from("--yaml"),
                OsString::from("--precision"),
                OsString::from("0.01"),
                OsString::from("--precision"),
                OsString::from("0.02"),
                OsString::from("--set"),
                OsString::from("--mset"),
                OsString::from("--setkeys"),
                OsString::from("id"),
                OsString::from("--setkeys"),
                OsString::from("name"),
                OsString::from("--opts"),
                OsString::from("[\"SET\"]"),
                OsString::from("--opts"),
                OsString::from("[{\"precision\":0.1}]")
            ]
        );
    }

    #[test]
    fn output_format_default_is_native() {
        assert_eq!(OutputFormat::default(), OutputFormat::Native);
    }
}
