use pinix::wrapper::command::NixCommand;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    NixCommand::from_args(std::env::args().skip(1))
        .exec_copycat()
        .await
}
