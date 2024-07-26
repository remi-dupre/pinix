use console::style;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;

use crate::action::{Action, BuildStepId, ResultFields, StartFields};
use crate::handlers::logs::LogHandler;
use crate::state::{Handler, HandlerResult, State};
use crate::style::{format_build_target, format_short_build_target, template_style, MultiBar};

static C_RUN: Lazy<String> = Lazy::new(|| style("-").blue().bright().to_string());

/// Min size of the package for a progressbar to be displayed
const MIN_PROGRESS_PAYLOAD: u64 = 10 * 1024 * 1024; // 1MB

fn get_style(size: u16) -> ProgressStyle {
    template_style(
        size,
        true,
        |size| match size {
            0..=50 => "Download {wide_msg}",
            51..=60 => "Download {wide_msg} {binary_bytes_per_sec:^12}",
            _ => "Download {wide_msg} {binary_bytes_per_sec:^12} {bytes:^12}",
        },
        |_| "[{prefix}]",
    )
}

pub fn handle_new_download(state: &mut State, action: &Action) -> anyhow::Result<HandlerResult> {
    if let Action::Start {
        start_type: StartFields::CopyPath { path, .. },
        id,
        ..
    } = action
    {
        state.plug(LogHandler::new(*id));

        state.plug(WaitForTransfer {
            copy_id: *id,
            path: path.to_string(),
        })
    };

    Ok(HandlerResult::Continue)
}

/// A new download was registered, waiting for corresponding transfer
struct WaitForTransfer {
    copy_id: BuildStepId,
    path: String,
}

impl Handler for WaitForTransfer {
    fn on_action(&mut self, state: &mut State, action: &Action) -> anyhow::Result<HandlerResult> {
        match action {
            Action::Start {
                start_type: StartFields::FileTransfer { .. },
                id,
                parent,
                ..
            } if *parent == self.copy_id => {
                state.plug(Transfer {
                    transfer_id: *id,
                    progress: None,
                    path: std::mem::take(&mut self.path),
                });

                state.plug(LogHandler::new(*id));
                Ok(HandlerResult::Close)
            }
            _ => Ok(HandlerResult::Continue),
        }
    }
}

/// Keep track of transfer
struct Transfer {
    transfer_id: BuildStepId,
    progress: Option<ProgressBar>,
    path: String,
}

impl Transfer {
    fn update_bar(&self, term_size: u16) {
        if let Some(progress) = &self.progress {
            let pos = progress.position();
            let exp = progress.length().unwrap_or(pos);

            progress.set_style(get_style(term_size));
            progress.set_prefix(
                MultiBar([("#", pos), (C_RUN.as_str(), exp - pos)])
                    .scale(u64::from(term_size) / 3)
                    .to_string(),
            )
        }
    }
}

impl Handler for Transfer {
    fn on_action(&mut self, state: &mut State, action: &Action) -> anyhow::Result<HandlerResult> {
        match action {
            Action::Result {
                id,
                fields: ResultFields::Progress { done, expected, .. },
            } if *id == self.transfer_id => {
                self.progress = self.progress.take().or_else(|| {
                    if *expected > 0 {
                        if *expected >= MIN_PROGRESS_PAYLOAD {
                            let pb = ProgressBar::new(*expected)
                                .with_style(get_style(state.term_size))
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

                self.update_bar(state.term_size);
                Ok(HandlerResult::Continue)
            }

            Action::Stop { id } if *id == self.transfer_id => {
                if state.cmd.args.show_downloads {
                    if let Some(progress) = &self.progress {
                        let msg_main = format!(
                            "{} Downloaded {}",
                            style("⭣").green(),
                            format_build_target(&self.path),
                        );

                        let msg_stats = style(format!(
                            " ({}, {:.0?})",
                            HumanBytes(progress.position()),
                            progress.duration(),
                        ))
                        .dim()
                        .to_string();

                        state.println(msg_main + &msg_stats)?;
                        progress.finish_and_clear();
                    }
                }

                self.update_bar(state.term_size);
                Ok(HandlerResult::Close)
            }

            _ => Ok(HandlerResult::Continue),
        }
    }

    fn on_resize(&mut self, state: &mut State) -> anyhow::Result<()> {
        if let Some(progress) = &self.progress {
            progress.set_style(get_style(state.term_size));
        }

        Ok(())
    }
}
