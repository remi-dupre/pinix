use console::style;
use indicatif::{HumanBytes, ProgressBar};

use crate::action::{Action, ActionType, BuildStepId, ResultFields, StartFields};
use crate::state::{Handler, HandlerResult, NewState};
use crate::style::{format_build_target, DOWNLOAD_STYLE, SPINNER_FREQ};

use super::logs::LogHandler;

pub fn handle_new_download(state: &mut NewState, action: &Action) -> HandlerResult {
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
    fn handle(&mut self, state: &mut NewState, action: &Action) -> HandlerResult {
        match action {
            Action::Start {
                action_type: ActionType::FileTransfer,
                id,
                parent,
                ..
            } if *parent == self.copy_id => {
                let progress = ProgressBar::new_spinner()
                    .with_style(DOWNLOAD_STYLE.clone())
                    .with_prefix("Download")
                    .with_message(format_build_target(&self.path));

                let progress = state.multi_progress.insert(0, progress);
                progress.enable_steady_tick(SPINNER_FREQ);

                state.plug(Transfering {
                    transfer_id: *id,
                    progress,
                });

                state.plug(LogHandler::new(*id));
                HandlerResult::Close
            }
            _ => HandlerResult::Continue,
        }
    }
}

/// Keep track of transfer
struct Transfering {
    transfer_id: BuildStepId,
    progress: ProgressBar,
}

impl Handler for Transfering {
    fn handle(&mut self, state: &mut NewState, action: &Action) -> HandlerResult {
        match action {
            Action::Result {
                action_type: ActionType::Build,
                id,
                fields: ResultFields::Progress([done, expected, ..]),
            } if *id == self.transfer_id => {
                self.progress.set_length(*expected);
                self.progress.set_position(*done);
                HandlerResult::Continue
            }

            Action::Stop { id } if *id == self.transfer_id => {
                let msg_main = format!(
                    "{} Downloaded {}",
                    style("⬇").green(),
                    self.progress.message()
                );

                let msg_stats = style(format!(
                    " ({}, {:.0?})",
                    HumanBytes(self.progress.position()),
                    self.progress.duration(),
                ))
                .dim()
                .to_string();

                state.println(msg_main + &msg_stats);
                self.progress.finish_and_clear();
                HandlerResult::Close
            }

            _ => HandlerResult::Continue,
        }
    }
}
