use std::env;

use pinix::commands::Program;
use pinix::nix_copycat;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    nix_copycat(Program::NixOsRebuild, env::args().skip(1)).await
}
