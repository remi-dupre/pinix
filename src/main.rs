pub mod action;
pub mod commands;
pub mod event;
pub mod handlers;
pub mod state;
pub mod style;

use std::process::{ExitStatus, Stdio};

use anyhow::Context;
use futures::FutureExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use self::commands::NixCommand;
use self::state::State;

fn run_from_stdin() {
    let mut state = State::new("Piped Output");

    for line in std::io::stdin().lines() {
        let line = line.expect("could not read line");

        if let Some(action_str) = line.strip_prefix("@nix ") {
            let action = serde_json::from_str(action_str).expect("invalid JSON in action");
            state.handle(&action);
        } else {
            state.println(line);
            continue;
        };
    }
}

async fn run_command_wrapped(cmd: &NixCommand) -> anyhow::Result<ExitStatus> {
    let mut state = State::new(&format!(
        "{} {}",
        cmd.program_str(),
        cmd.params_unwrapped().collect::<Vec<_>>().join(" ")
    ));

    let mut child = Command::new(cmd.program_str())
        .args(cmd.params_wrapped())
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn command")?;

    let stdout = child
        .stdout
        .as_mut()
        .context("could not read child command output")?;

    let stderr = child
        .stderr
        .as_mut()
        .context("could not read child command output")?;

    let mut stdout = BufReader::new(stdout).lines();
    let mut stderr = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            Some(line) = stdout.next_line().map(Result::transpose) => {
                let line = line.expect("failed to read stdout");
                state.println(line);
            }

            Some(line) = stderr.next_line().map(Result::transpose) => {
                let line = line.expect("failed to read stderr");

                if let Some(action_str) = line.strip_prefix("@nix ") {
                    let action = serde_json::from_str(action_str).expect("invalid JSON in action");
                    state.handle(&action);
                } else {
                    state.println(line);
                };
            }

            else => {
                break;
            }
        }
    }

    child.wait().await.context("child command failed")
}

async fn run_command_unwrapped(cmd: &NixCommand) -> anyhow::Result<ExitStatus> {
    Command::new(cmd.program_str())
        .args(cmd.params_unwrapped())
        .spawn()
        .context("failed to spawn repl command")?
        .wait()
        .await
        .context("replcommand failed")
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let Some(command) = NixCommand::parse_from_args() else {
        run_from_stdin();
        return Ok(());
    };

    let exit_code = run_command_wrapped(&command).await?;

    if !exit_code.success() {
        anyhow::bail!("command finished with exit code {exit_code}");
    }

    if command.is_repl() {
        run_command_unwrapped(&command).await?;
    }

    Ok(())
}
