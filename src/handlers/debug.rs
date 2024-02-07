use std::cmp::max;

use indicatif::{ProgressBar, ProgressFinish};

use crate::action::Action;
use crate::state::{Handler, HandlerResult, State};
use crate::style::{PROGRESS_STYLE, SPINNER_FREQ};

pub struct DebugHandler {
    progress: ProgressBar,
}

impl DebugHandler {
    pub fn new(state: &State) -> Self {
        let progress = ProgressBar::new_spinner()
            .with_style(PROGRESS_STYLE.clone())
            .with_finish(ProgressFinish::Abandon)
            .with_prefix("Debug");

        let progress = state.multi_progress.add(progress);
        progress.enable_steady_tick(SPINNER_FREQ);
        DebugHandler { progress }
    }
}

impl Handler for DebugHandler {
    fn handle(&mut self, state: &mut State, _action: &Action) -> HandlerResult {
        let handlers_len = state.handlers_len as _;

        self.progress
            .set_length(max(handlers_len, self.progress.length().unwrap_or(0)));

        self.progress.set_position(handlers_len);
        HandlerResult::Continue
    }
}
