use console::style;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};

use crate::action::{Action, ActionType, BuildStepId, ResultFields, StartFields};
use crate::handlers::logs::LogHandler;
use crate::state::{Handler, HandlerResult, State};
use crate::style::{format_build_target, format_short_build_target, template_style};

/// Min size of the package for a progressbar to be displayed
const MIN_PROGRESS_PAYLOAD: u64 = 10 * 1024 * 1024; // 1MB

fn build_style(size: u16) -> ProgressStyle {
    template_style(
        size,
        true,
        |size| match size {
            0..=50 => "{prefix} {wide_msg}",
            51..=60 => "{prefix} {wide_msg} {binary_bytes_per_sec:^12}",
            _ => "{prefix} {wide_msg} {binary_bytes_per_sec:^12} {bytes:^12}",
        },
        |size| format!("[{{bar:{size}}}]"),
    )
}

pub fn handle_new_download(state: &mut State, action: &Action) -> HandlerResult {
    if let Action::Start {
        action_type: ActionType::CopyPath,
        id,
        fields: StartFields::Copy([path, _, _]),
        ..
    } = action
    {
        state.plug(LogHandler::new(*id));

        state.plug(WaitForTransfer {
            copy_id: *id,
            path: path.to_string(),
        })
    };

    HandlerResult::Continue
}

/// A new download was registered, waiting for corresponding transfer
struct WaitForTransfer {
    copy_id: BuildStepId,
    path: String,
}

impl Handler for WaitForTransfer {
    fn handle(&mut self, state: &mut State, action: &Action) -> HandlerResult {
        match action {
            Action::Start {
                action_type: ActionType::FileTransfer,
                id,
                parent,
                ..
            } if *parent == self.copy_id => {
                state.plug(Transfering {
                    transfer_id: *id,
                    progress: None,
                    path: std::mem::take(&mut self.path),
                });

                state.plug(LogHandler::new(*id));
                HandlerResult::Close
            }
            _ => HandlerResult::Continue,
        }
    }

    fn resize(&mut self, _state: &mut State, _size: u16) {}
}

/// Keep track of transfer
struct Transfering {
    transfer_id: BuildStepId,
    progress: Option<ProgressBar>,
    path: String,
}

impl Handler for Transfering {
    fn handle(&mut self, state: &mut State, action: &Action) -> HandlerResult {
        match action {
            Action::Result {
                action_type: ActionType::Build,
                id,
                fields: ResultFields::Progress([done, expected, ..]),
            } if *id == self.transfer_id => {
                self.progress = self.progress.take().or_else(|| {
                    if *expected > 0 {
                        if *expected >= MIN_PROGRESS_PAYLOAD {
                            let pb = ProgressBar::new(*expected)
                                .with_style(build_style(state.term_size))
                                .with_prefix("Download")
                                .with_message(format_short_build_target(&self.path));
                            Some(state.add(pb))
                        } else {
                            Some(ProgressBar::hidden())
                        }
                    } else {
                        None
                    }
                });

                if let Some(progress) = &self.progress {
                    progress.set_length(*expected);
                    progress.set_position(*done);
                }

                HandlerResult::Continue
            }

            Action::Stop { id } if *id == self.transfer_id => {
                if state.cmd.args.log_downloads {
                    if let Some(progress) = &self.progress {
                        let msg_main = format!(
                            "{} Downloaded {}",
                            style("⬇").green(),
                            format_build_target(&self.path),
                        );

                        let msg_stats = style(format!(
                            " ({}, {:.0?})",
                            HumanBytes(progress.position()),
                            progress.duration(),
                        ))
                        .dim()
                        .to_string();

                        state.println(msg_main + &msg_stats);
                        progress.finish_and_clear();
                    }
                }

                HandlerResult::Close
            }

            _ => HandlerResult::Continue,
        }
    }

    fn resize(&mut self, _state: &mut State, size: u16) {
        if let Some(progress) = &self.progress {
            progress.set_style(build_style(size));
        }
    }
}
