use std::iter;
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::Context;
use clap::{CommandFactory, Parser};
use console::style;
use tokio::process;

use crate::state::monitor_logs;

use super::stream::MergedStreams;

#[derive(Debug, clap::Parser)]
#[command(
    disable_help_flag = true,
    allow_hyphen_values = true,
    trailing_var_arg = true
)]
/// Wrap a Nix command to display rich logs while it is running.
pub struct Args {
    #[clap(long = "pix-help", help = "Display this help message")]
    pub help: bool,

    #[arg(long = "pix-debug", help = "Display a debug bar")]
    pub debug: bool,

    #[arg(
        long = "pix-log-downloads",
        help = "Display a log line when a download is finished"
    )]
    pub log_downloads: bool,

    #[arg(long = "pix-record", help = "Save timestamped logs to a file")]
    pub record: Option<PathBuf>,

    #[clap(help = "Arguments forwared to actual Nix command")]
    pub ext: Vec<String>,
}

#[derive(Debug)]
pub enum WrappedProgram {
    Nix,
    NixCollectGarbage,
    NixOsRebuild,
    NixShell,
    Unknown(String),
}

impl WrappedProgram {
    pub fn as_str(&self) -> &str {
        match self {
            WrappedProgram::Nix => "nix",
            WrappedProgram::NixCollectGarbage => "nix-collect-garbage",
            WrappedProgram::NixOsRebuild => "nixos-rebuild",
            WrappedProgram::NixShell => "nix-shell",
            WrappedProgram::Unknown(path) => path.as_str(),
        }
    }
}

impl From<String> for WrappedProgram {
    fn from(value: String) -> Self {
        match value.as_str() {
            "nix" => Self::Nix,
            "nix-collect-garbage" => Self::NixCollectGarbage,
            "nixos-rebuild" => Self::NixOsRebuild,
            "nix-shell" => Self::NixShell,
            _ => Self::Unknown(value),
        }
    }
}

impl std::fmt::Display for WrappedProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub struct NixCommand {
    pub program: WrappedProgram,
    pub args: Args,
}

impl NixCommand {
    pub fn params_unwrapped(&self) -> impl Iterator<Item = &'_ str> + '_ {
        self.args.ext.iter().map(String::as_str)
    }

    pub fn params_wrapped(&self) -> impl Iterator<Item = &'_ str> + '_ {
        self.extra_params()
            .map(|s| s as _)
            .chain(self.args.ext.iter().map(String::as_str))
    }

    pub fn is_repl(&self) -> bool {
        matches!(
            (&self.program, self.args.ext.first().map(String::as_str)),
            (WrappedProgram::NixShell, _)
                | (WrappedProgram::Nix, Some("repl" | "develop" | "shell"))
        )
    }

    fn extra_params(&self) -> impl Iterator<Item = &'static str> + '_ {
        let required: &[(&str, &[_])] = match (&self.program, self.args.ext.as_slice()) {
            (WrappedProgram::Nix | WrappedProgram::NixOsRebuild, &[..]) => &[
                ("--print-build-logs", &[]),
                ("--log-format", &["internal-json"]),
            ],
            (WrappedProgram::NixCollectGarbage | WrappedProgram::NixShell, &[..]) => {
                &[("--log-format", &["internal-json"])]
            }
            (WrappedProgram::Unknown(_), &[..]) => &[],
        };

        required
            .iter()
            .filter(move |(flag, _)| self.args.ext.iter().all(|arg| arg != flag))
            .flat_map(move |(flag, vals)| iter::once(flag).chain(*vals))
            .copied()
    }
}

impl NixCommand {
    pub fn from_program_and_args(
        program: WrappedProgram,
        args: impl Iterator<Item = String>,
    ) -> Self {
        let args = Args::parse_from(iter::once(program.to_string()).chain(args));

        if args.help {
            Args::command()
                .print_help()
                .expect("failed to display help message");

            std::process::exit(0);
        }

        Self { program, args }
    }

    pub fn from_args(args: impl Iterator<Item = String>) -> Self {
        let mut cmd = Self::from_program_and_args(WrappedProgram::Unknown(String::new()), args);

        if cmd.args.ext.is_empty() {
            eprintln!(
                "{}: No program to execute",
                style("error").bright().red().bold()
            );

            Args::command()
                .print_help()
                .expect("failed to display help message");

            std::process::exit(1);
        };

        let program_str = cmd.args.ext.remove(0);
        cmd.program = WrappedProgram::Unknown(program_str);
        cmd
    }

    pub async fn exec_copycat(&self) -> anyhow::Result<()> {
        let mut child = process::Command::new(self.program.as_str())
            .args(self.params_wrapped())
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("failed to spawn command")?;

        let logs_stream =
            MergedStreams::new(&mut child).context("could not pipe command output")?;

        monitor_logs(self, logs_stream).await?;
        let exit_code = child.wait().await.context("child command failed")?;

        if !exit_code.success() {
            std::process::exit(exit_code.code().context("unknown exit code")?);
        }

        if self.is_repl() {
            process::Command::new(self.program.as_str())
                .args(self.params_unwrapped())
                .spawn()
                .context("failed to spawn repl command")?
                .wait()
                .await
                .context("replcommand failed")?;
        }

        Ok(())
    }
}
