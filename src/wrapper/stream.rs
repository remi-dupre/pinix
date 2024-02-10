use anyhow::Context;
use futures::{FutureExt, TryStream};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

pub enum WrappedLine {
    StdOut(String),
    StdErr(String),
}

pub trait StreamedPipes<'a>: TryStream<Ok = WrappedLine, Error = anyhow::Error> + 'a {}
impl<'a, T> StreamedPipes<'a> for T where T: TryStream<Ok = WrappedLine, Error = anyhow::Error> + 'a {}

pub fn stream_child_output(child: &mut Child) -> anyhow::Result<impl StreamedPipes<'_>> {
    let stdout = child
        .stdout
        .as_mut()
        .context("could not read child command output")?;

    let stderr = child
        .stderr
        .as_mut()
        .context("could not read child command output")?;

    let stdout = BufReader::new(stdout).lines();
    let stderr = BufReader::new(stderr).lines();

    Ok(futures::stream::try_unfold(
        (stdout, stderr),
        |(mut stdout, mut stderr)| async move {
            let line = tokio::select! {
                Some(line) = stdout.next_line().map(Result::transpose) => {
                    WrappedLine::StdOut(line.context("could not read child's stdout")?)
                }
                Some(line) = stderr.next_line().map(Result::transpose) => {
                    WrappedLine::StdErr(line.context("could not read child's stderr")?)
                }
                else => {
                    return Ok(None)
                }
            };

            Ok(Some((line, (stdout, stderr))))
        },
    ))
}
