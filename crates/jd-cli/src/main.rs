//! Command-line interface for the Rust port of the Go `jd` tool.
//!
//! The current milestone provides a minimal stub that supports `--help`
//! and `--version` so that smoke tests can validate the workspace
//! scaffolding. Subsequent milestones will flesh out the full flag
//! surface and functionality.

use std::ffi::OsString;
use std::io::{self, Write};

use anyhow::Result;
use clap::{ArgAction, Parser};

const VERSION_BANNER: &str = concat!("jd version ", env!("CARGO_PKG_VERSION"));

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
}

fn main() {
    if let Err(err) = try_main() {
        let _ = writeln!(io::stderr(), "{err}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let args = canonicalize_args(std::env::args_os());
    let cli = Cli::parse_from(args);

    if cli.version {
        println!("{VERSION_BANNER}");
        return Ok(());
    }

    Ok(())
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
            Some("-help") => {
                canonicalized.push(OsString::from("--help"));
            }
            Some("-version") => {
                canonicalized.push(OsString::from("--version"));
            }
            _ => canonicalized.push(arg),
        }
    }
    canonicalized
}

#[cfg(test)]
mod tests {
    use super::canonicalize_args;
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
}
