use std::env;

use pinix::commands::NixCommand;
use pinix::{run_command_unwrapped, run_command_wrapped, run_from_stdin};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let Some(command) = NixCommand::from_args(env::args().skip(1)) else {
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
