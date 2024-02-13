use std::collections::HashMap;

use console::style;
use indexmap::IndexMap;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};

use crate::action::{Action, ActionType, BuildStepId, ResultFields, StartFields};
use crate::state::{Handler, HandlerResult, State};
use crate::style::{format_short_build_target, template_style, MultiBar};

fn get_style(size: u16, done: u64, expected: u64, running: u64) -> ProgressStyle {
    template_style(
        size,
        true,
        |size| match size {
            0..=50 => "{prefix} {wide_msg}",
            51..=60 => "{prefix} {wide_msg} {binary_bytes_per_sec:^12}",
            _ => "{prefix} {wide_msg} {binary_bytes_per_sec:^12} {bytes:^12}",
        },
        |size| {
            let size = u64::from(size);

            let adv1 = (size * done + expected / 2)
                .checked_div(expected)
                .unwrap_or(0);

            let adv2 = (size * running + expected / 2)
                .checked_div(expected)
                .unwrap_or(0);

            let c_pos = "#";
            let c_run = style("-").blue().bright().to_string();
            let bar = MultiBar([(c_pos, adv1), (&c_run, adv2 - adv1), (" ", size - adv2)]);
            format!("[{bar}]")
        },
    )
}

pub fn handle_new_downloads_group(state: &mut State, action: &Action) -> HandlerResult {
    if let Action::Start {
        action_type: ActionType::CopyPaths,
        id,
        ..
    } = action
    {
        let handler = DownloadsGroup::new(*id, state);
        state.plug(handler);
    }

    HandlerResult::Continue
}

struct DownloadsGroup {
    id: BuildStepId,
    progress: ProgressBar,
    current_copies: IndexMap<BuildStepId, String>,
    state_copy: HashMap<BuildStepId, [u64; 4]>,
    state_transfer: HashMap<BuildStepId, [u64; 4]>,
    state_self: [u64; 4],
    max_copy: u64,
    max_transfer: u64,
}

impl DownloadsGroup {
    fn new(id: BuildStepId, state: &mut State) -> Self {
        let progress = ProgressBar::new_spinner().with_style(get_style(state.term_size, 0, 0, 0));

        Self {
            id,
            progress: state.add(progress),
            current_copies: IndexMap::new(),
            state_copy: HashMap::new(),
            state_transfer: HashMap::new(),
            state_self: [0; 4],
            max_copy: 0,
            max_transfer: 0,
        }
    }

    fn get_done(&self) -> u64 {
        self.state_transfer.values().map(|&[done, ..]| done).sum()
    }

    fn get_running(&self) -> u64 {
        self.state_transfer
            .values()
            .map(|&[_, expected, ..]| expected)
            .sum()
    }

    fn get_unpacked(&self) -> u64 {
        self.state_copy.values().map(|&[done, ..]| done).sum()
    }

    fn redraw(&self, size: u16) {
        let message = self
            .current_copies
            .values()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ");

        self.progress.set_style(get_style(
            size,
            self.get_done(),
            self.max_transfer,
            self.get_running(),
        ));

        self.progress.set_prefix(format!(
            "Downloaded ({}/{})",
            self.state_self[0], self.state_self[1],
        ));

        self.progress.set_position(self.get_done());
        self.progress.set_length(self.max_transfer);
        self.progress.set_message(message);
    }
}

impl Handler for DownloadsGroup {
    fn handle(&mut self, state: &mut State, action: &Action) -> HandlerResult {
        match action {
            Action::Start {
                action_type: ActionType::CopyPath,
                id,
                fields: StartFields::Copy([path, _, _]),
                ..
            } => {
                self.state_copy.insert(*id, [0; 4]);
                self.current_copies
                    .insert(*id, format_short_build_target(path));
            }

            Action::Start {
                action_type: ActionType::FileTransfer,
                id,
                ..
            } => {
                self.state_transfer.insert(*id, [0; 4]);
            }

            Action::Result {
                action_type: ActionType::Build,
                id,
                fields: ResultFields::Progress(state),
            } => {
                if *id == self.id {
                    self.state_self = *state;
                }

                if let Some(copy) = self.state_copy.get_mut(id) {
                    *copy = *state;
                }

                if let Some(transfer) = self.state_transfer.get_mut(id) {
                    *transfer = *state;
                }
            }

            Action::Result {
                action_type: ActionType::OptimiseStore,
                fields: ResultFields::Realise(ActionType::CopyPath, max_copy),
                ..
            } => {
                self.max_copy = *max_copy;
            }

            Action::Result {
                action_type: ActionType::OptimiseStore,
                fields: ResultFields::Realise(ActionType::FileTransfer, max_transfer),
                ..
            } => {
                self.max_transfer = *max_transfer;
            }

            Action::Stop { id } if *id == self.id => {
                if self.state_self[0] > 0 {
                    let msg_main = format!(
                        "{} Downloaded {} derivations",
                        style("⬇").green(),
                        self.state_self[0],
                    );

                    let msg_stats = style(format!(
                        " ({} downloaded, {} unpacked, {:.0?})",
                        HumanBytes(self.get_done()),
                        HumanBytes(self.get_unpacked()),
                        self.progress.duration(),
                    ))
                    .dim()
                    .to_string();

                    state.println(msg_main + &msg_stats);
                    self.progress.finish_and_clear();

                    return HandlerResult::Close;
                }
            }

            Action::Stop { id } => {
                self.current_copies.shift_remove(id);
            }

            _ => {}
        }

        self.redraw(state.term_size);
        HandlerResult::Continue
    }

    fn resize(&mut self, _state: &mut State, size: u16) {
        self.redraw(size);
    }
}
