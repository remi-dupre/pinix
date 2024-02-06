use std::rc::Rc;

use console::style;
use indicatif::{HumanBytes, ProgressBar};

use crate::action::BuildStepId;
use crate::event::EventTriggerResult;
use crate::style::{format_build_target, DOWNLOAD_STYLE, SPINNER_FREQ};

use super::state::State;

pub struct StateDownload;

impl StateDownload {
    pub fn connect(state: Rc<State>) {
        state.events_download_start.connect((), {
            let state = state.clone();

            move |(id, path)| {
                Self::init_download(state.clone(), *id, path.to_string());
                EventTriggerResult::Continue
            }
        })
    }

    fn init_download(state: Rc<State>, id: BuildStepId, path: String) {
        state.events_transfer_start.connect(id, {
            let state = state.clone();

            move |transfer_id| {
                Self::start_download(state.clone(), path.clone(), *transfer_id);
                EventTriggerResult::Unregister
            }
        })
    }

    fn start_download(state: Rc<State>, path: String, transfer_id: BuildStepId) {
        let progress = ProgressBar::new_spinner()
            .with_style(DOWNLOAD_STYLE.clone())
            .with_prefix("Download")
            .with_message(format_build_target(&path));

        progress.enable_steady_tick(SPINNER_FREQ);
        let progress = state.main_progress.add(progress);
        let progress = Rc::new(progress);
        let progress_weak = Rc::downgrade(&progress);

        state
            .events_status
            .connect(transfer_id, move |&[done, expected, ..]| {
                let Some(progress) = progress_weak.upgrade() else {
                    return EventTriggerResult::Unregister;
                };

                progress.set_length(expected);
                progress.set_position(done);
                EventTriggerResult::Continue
            });

        state.events_stop.connect(transfer_id, {
            let state = state.clone();

            move |&()| {
                let msg_main = format!("{} Downloaded {}", style("⬇").green(), progress.message());

                let msg_stats = style(format!(
                    " ({}, {:.0?})",
                    HumanBytes(progress.position()),
                    progress.duration(),
                ))
                .dim()
                .to_string();

                state
                    .main_progress
                    .println(msg_main + &msg_stats)
                    .expect("couldn't print line");

                progress.finish_and_clear();
                EventTriggerResult::Unregister
            }
        });
    }
}
