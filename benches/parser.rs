use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use divan::Bencher;
use pinix::action::Action;
use pinix::parse::action_raw::RawAction;

fn main() {
    divan::main();
}

fn load_example(example: &str) -> Vec<String> {
    let path: PathBuf = ["examples", example].iter().collect();
    let file = File::open(path).expect("could not open example");
    let reader = BufReader::new(file);

    reader
        .lines()
        .filter_map(|line| {
            let line = line.expect("could not read line");
            let msg = line.splitn(3, ' ').nth(2)?.strip_prefix("@nix ")?;
            Some(msg.to_string())
        })
        .collect()
}

// Register a `fibonacci` function and benchmark it over multiple cases.
#[divan::bench(args = ["nix-shell.rec", "nixos-rebuild.rec"])]
fn parse_raw(bencher: Bencher, example: &str) {
    let example = load_example(example);

    bencher
        .with_inputs(|| &example)
        .counter(divan::counter::ItemsCount::new(example.len()))
        .bench_refs(|lines| {
            lines
                .iter()
                .inspect(|line| {
                    let _: RawAction = serde_json::from_str(line).expect("invalid line");
                })
                .count()
        })
}

#[divan::bench(args = ["nix-shell.rec", "nixos-rebuild.rec"])]
fn parse(bencher: Bencher, example: &str) {
    let example = load_example(example);

    bencher
        .with_inputs(|| &example)
        .counter(divan::counter::ItemsCount::new(example.len()))
        .bench_refs(|lines| {
            lines
                .iter()
                .inspect(|line| {
                    let raw: RawAction = serde_json::from_str(line).expect("invalid line");
                    let _: Action = raw.try_into().unwrap();
                })
                .count()
        })
}
