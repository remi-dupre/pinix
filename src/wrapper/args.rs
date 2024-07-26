use std::path::PathBuf;

use crate::util::toml_ext::TomlBuilder;
use crate::wrapper::command::WrappedProgram;

#[derive(Debug, clap::Parser)]
#[command(
    allow_hyphen_values = true,
    disable_help_flag = true,
    disable_version_flag = true,
    trailing_var_arg = true
)]
/// Wrap a Nix command to display rich logs while it is running.
pub struct Args {
    #[clap(long = "pix-help", help = "Display this help message")]
    pub help: bool,

    #[arg(
        long = "pix-command",
        help = "Specify the nix command that must be run"
    )]
    pub command: Option<WrappedProgram>,

    #[arg(long = "pix-debug", help = "Display a debug bar")]
    pub debug: Option<bool>,

    #[arg(
        long = "pix-summary-download",
        help = "Display a summary line when a download is finished"
    )]
    pub summary_download: Option<bool>,

    #[arg(
        long = "pix-log-window-size",
        help = "Size of the window displaying build logs"
    )]
    pub log_window_size: Option<u32>,

    #[arg(
        long = "pix-log-history",
        help = "Restrict the size of the log history of each build"
    )]
    pub log_history: Option<Option<usize>>,

    #[arg(
        long = "pix-log-history-failure",
        help = "Restrict the size of the log history of each build in case of failure"
    )]
    pub log_history_failure: Option<Option<usize>>,

    #[arg(long = "pix-record", help = "Save timestamped logs to a file")]
    pub record: Option<PathBuf>,

    #[clap(help = "Arguments forwared to actual Nix command")]
    pub ext: Vec<String>,
}

impl Args {
    pub fn as_toml_overrides(&self) -> toml::Value {
        TomlBuilder::default()
            .with_opt(["debug"], self.debug)
            .with_opt(["summary", "download"], self.summary_download)
            .with_opt(["log-window", "size"], self.log_window_size)
            .build()
    }
}
