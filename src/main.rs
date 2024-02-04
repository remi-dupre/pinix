use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;
use std::time::Duration;

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Clone, Copy, Debug, Deserialize_repr)]
#[repr(u8)]
pub enum ActionType {
    Unknown = 0,
    CopyPath = 100,
    FileTransfer = 101,
    Realise = 102,
    CopyPaths = 103,
    Builds = 104,
    Build = 105,
    OptimiseStore = 106,
    VerifyPaths = 107,
    Substitute = 108,
    QueryPathInfo = 109,
    PostBuildHook = 110,
    BuildWaiting = 111,
}

impl Default for ActionType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Eq, PartialEq, Hash)]
pub struct BuildStepId(u64);

impl Deref for BuildStepId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for BuildStepId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<BuildStepId> for u64 {
    fn from(val: BuildStepId) -> Self {
        val.0
    }
}

#[derive(Debug, Default)]
pub struct BuildStep {
    step_id: BuildStepId,
    parent: BuildStepId,
    children: Vec<BuildStepId>,
    action_type: ActionType,
    text: String,
    /// Has it already logged at least a line yet?
    has_logged: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Fields {
    Progress([u64; 4]),
    Whatever([u64; 2]),
    Msg([String; 1]),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Action<'a> {
    Msg {
        level: u8, // TODO
        msg: &'a str,
    },
    Start {
        #[serde(rename = "type")]
        action_type: ActionType,
        id: BuildStepId,
        level: u8, // TODO
        parent: BuildStepId,
        text: &'a str,
    },
    Result {
        #[serde(rename = "type")]
        action_type: ActionType,
        id: BuildStepId,
        fields: Fields,
    },
    Stop {
        id: BuildStepId,
    },
}

fn build_tick_strings() -> Vec<String> {
    let mut tick_strings: Vec<_> = "⠁⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉⠈⠈"
        .chars()
        .map(|c| style(c).blue().to_string())
        .collect();

    tick_strings.push(style("✓").green().to_string());
    tick_strings
}

fn main() {
    let progress = MultiProgress::new();
    let mut steps = HashMap::new();
    let mut step_bars = HashMap::new();
    // let mut message_bars = Vec::new();

    for line in std::io::stdin().lines() {
        let line = line.expect("could not read line");

        if let Some(update) = line.strip_prefix("@nix ") {
            if let Ok(val) = serde_json::from_str::<Action>(update) {
                match val {
                    Action::Msg { level: _, msg } => {
                        progress.println(msg).expect("could not write line");
                    }

                    Action::Start {
                        action_type,
                        id,
                        level: _,
                        parent,
                        text,
                    } => {
                        steps.insert(
                            id,
                            BuildStep {
                                step_id: id,
                                parent,
                                children: vec![],
                                action_type,
                                text: text.to_string(),
                                has_logged: false,
                            },
                        );

                        let tick_strings = build_tick_strings();
                        let tick_strings: Vec<_> =
                            tick_strings.iter().map(|s| s.as_str()).collect();

                        let progress_bar = match action_type {
                            ActionType::CopyPath
                            | ActionType::CopyPaths
                            | ActionType::Substitute
                            | ActionType::Realise => {
                                continue
                            },

                            ActionType::FileTransfer=> {
                                ProgressBar::new(0)
                                .with_style(
                                    ProgressStyle::with_template(
                                        "{spinner} {wide_msg} {binary_bytes_per_sec:<14} {bytes:<12} [{bar:30}] {elapsed:>4}",
                                    ).unwrap()
                                    .progress_chars("## ")
                                    .tick_strings(tick_strings.as_slice()),
                                )
                            }
                            ActionType::Builds => {
                                ProgressBar::new(0)
                                .with_style(
                                    ProgressStyle::with_template(
                                        "{spinner} {wide_msg} {pos}/{len} [{bar:30}] {elapsed:>4}",
                                    ).unwrap()
                                    .progress_chars("## ")
                                    .tick_strings(tick_strings.as_slice()),
                                )
                            }
                            _ => {
                                ProgressBar::new_spinner()
                                    .with_style(
                                        ProgressStyle::with_template(
                                            "{spinner} {wide_msg} {elapsed}"
                                        ).unwrap()
                                        .tick_strings(&tick_strings),
                                    )
                            }
                        };

                        progress_bar.enable_steady_tick(Duration::from_millis(100));

                        let progress_bar = progress_bar.with_message(text.to_string());

                        let progress_bar = {
                            if let Some(parent_bar) = step_bars.get(&parent) {
                                progress.insert_before(&parent_bar, progress_bar)
                            } else {
                                progress.add(progress_bar)
                            }
                        };

                        step_bars.insert(id, progress.add(progress_bar));
                    }

                    Action::Result {
                        action_type: _,
                        id,
                        fields,
                    } => {
                        if let Some(progress_bar) = step_bars.get(&id) {
                            match fields {
                                Fields::Progress([done, expected, ..]) => {
                                    progress_bar.set_length(expected);
                                    progress_bar.set_position(done);
                                }
                                Fields::Whatever(_) => todo!(),
                                Fields::Msg([msg]) => {
                                    // let step = steps.get_mut(&id).unwrap();
                                    // step.has_logged = true;
                                    //
                                    // let bar = ProgressBar::new_spinner()
                                    //     .with_style(ProgressStyle::with_template("{msg}").unwrap())
                                    //     .with_message(style(format!("└ {msg}")).dim().to_string());
                                    //
                                    // let bar = progress.add(bar);
                                    // bar.tick();
                                    // message_bars.push(bar);
                                    // std::thread::sleep(Duration::from_millis(2000));

                                    // .finish_with_message(
                                    //     ,
                                    // );

                                    // message_bar.tick();
                                }
                            }
                        }
                        // ActionType::FileTransfer | ActionType::Build | ActionType::Builds => {
                        //     step_bars[&id].set_position(fields[0]);
                        //     step_bars[&id].set_length(fields[1]);
                        // }
                        // _ => {
                        //     // println!("{action_type:?}");
                        // }
                    }

                    Action::Stop { id } => {
                        if let Some(progress_bar) = step_bars.remove(&id) {
                            progress.remove(&progress_bar);
                            let progress_bar = progress.insert(0, progress_bar);
                            progress_bar.finish();
                            progress.println("hello");
                        }
                    }
                }
            } else {
                progress
                    .println(format!("/!\\ Could not deserialize {update:?}"))
                    .unwrap();
            }
        } else {
            progress.println(line).expect("could not write line");
        }

        std::thread::sleep(Duration::from_micros(10));
    }
}
