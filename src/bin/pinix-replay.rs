use std::fs::File;

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::Context;
use clap::Parser;
use pinix::wrapper::stream::OutputStream;

#[derive(Debug, clap::Parser)]
/// Mimic an actual Nix command by reading a record file. Which is useful to
/// demo or test pix.
pub struct Args {
    #[arg(
        long,
        short,
        default_value = "1.0",
        help = "Speedup replay by given factor"
    )]
    pub factor: f64,

    #[arg(
        long,
        short,
        default_value = "0.0",
        help = "Race the begining of the record for a given amount of seconds"
    )]
    pub skip: f64,

    #[arg(help = "Path to a record file")]
    pub path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let start_time = Instant::now();
    let file = File::open(args.path).context("could not open record file")?;
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    for line in BufReader::new(file).lines() {
        let line = line.context("could not read line from file")?;
        let mut cols = line.splitn(3, ' ');

        let output: OutputStream = cols.next().context("missing output column")?.parse()?;

        let delay: u128 = cols
            .next()
            .context("missing delay column")?
            .parse()
            .context("invalid delay")?;

        let delay = (1000. * (delay as f64 - 1000. * args.skip).clamp(0., f64::INFINITY)
            / args.factor) as u128;

        let line = cols.next().unwrap_or("");
        let to_wait = delay.saturating_sub(start_time.elapsed().as_micros());
        sleep(Duration::from_micros(to_wait as _));

        match output {
            OutputStream::StdOut => {
                stdout
                    .write_all(format!("{line}\n").as_bytes())
                    .context("couldn't write to stdout")?;

                stdout.flush().context("couldn't flush stdout")?;
            }
            OutputStream::StdErr => {
                stderr
                    .write_all(format!("{line}\n").as_bytes())
                    .context("couldn't write to stderr")?;

                stderr.flush().context("couldn't flush stderr")?;
            }
        }
    }

    Ok(())
}
