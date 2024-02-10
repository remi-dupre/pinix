use std::str::FromStr;

use anyhow::Context;
use futures::{FutureExt, TryStream};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

pub enum OutputStream {
    StdOut,
    StdErr,
}

impl OutputStream {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputStream::StdOut => "stdout",
            OutputStream::StdErr => "stderr",
        }
    }
}

impl FromStr for OutputStream {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stdout" => Ok(Self::StdOut),
            "stderr" => Ok(Self::StdErr),
            _ => anyhow::bail!("unknown output type `{s}`"),
        }
    }
}

pub trait StreamedPipes<'a>:
    TryStream<Ok = (OutputStream, String), Error = anyhow::Error> + 'a
{
}

impl<'a, T> StreamedPipes<'a> for T where
    T: TryStream<Ok = (OutputStream, String), Error = anyhow::Error> + 'a
{
}

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
            let res = tokio::select! {
                Some(line) = stdout.next_line().map(Result::transpose) => {
                    (OutputStream::StdOut, line.context("could not read child's stdout")?)
                }
                Some(line) = stderr.next_line().map(Result::transpose) => {
                    (OutputStream::StdErr, line.context("could not read child's stderr")?)
                }
                else => {
                    return Ok(None)
                }
            };

            Ok(Some((res, (stdout, stderr))))
        },
    ))
}
