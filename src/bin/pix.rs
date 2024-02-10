use pinix::wrapper::command::{NixCommand, WrappedProgram};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    NixCommand::from_program_and_args(WrappedProgram::Nix, std::env::args().skip(1))
        .exec_copycat()
        .await
}
