use std::str::FromStr;

use anyhow::Context;
use futures::FutureExt;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdout};

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

/// Cancellable way to fetch lines from a reader
struct BorrowLines<R: AsyncRead + Unpin> {
    reader: BufReader<R>,
    buffer: Vec<u8>,
    yielded: bool,
}

impl<R: AsyncRead + Unpin> BorrowLines<R> {
    fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            buffer: Vec::new(),
            yielded: false,
        }
    }

    async fn next_line(&mut self) -> anyhow::Result<Option<&[u8]>> {
        if self.yielded {
            self.yielded = false;
            self.buffer.clear();
        }

        self.reader.read_until(b'\n', &mut self.buffer).await?;

        if self.buffer.is_empty() {
            Ok(None)
        } else {
            assert_eq!(self.buffer.last(), Some(&b'\n'));
            self.yielded = true;
            Ok(Some(self.buffer.as_slice()))
        }
    }
}

pub struct MergedStreams<'c> {
    stdout: BorrowLines<&'c mut ChildStdout>,
    stderr: BorrowLines<&'c mut ChildStderr>,
}

impl<'c> MergedStreams<'c> {
    pub fn new(child: &'c mut Child) -> anyhow::Result<Self> {
        let stdout = child
            .stdout
            .as_mut()
            .context("could not read child command output")?;

        let stderr = child
            .stderr
            .as_mut()
            .context("could not read child command output")?;

        Ok(Self {
            stdout: BorrowLines::new(stdout),
            stderr: BorrowLines::new(stderr),
        })
    }

    pub async fn next_line(&mut self) -> anyhow::Result<Option<(OutputStream, &[u8])>> {
        tokio::select! {
            Some(line) = self.stdout.next_line().map(Result::transpose) => {
                let line = line?;
                Ok(Some((OutputStream::StdOut, line)))
            }
            Some(line) = self.stderr.next_line().map(Result::transpose) => {
                let line = line?;
                Ok(Some((OutputStream::StdErr, line)))
            }
            else => {
                Ok(None)
            }
        }
    }
}
