use std::cmp::max;

use indicatif::{HumanCount, ProgressBar, ProgressFinish, ProgressStyle};

use crate::action::Action;
use crate::state::{Handler, HandlerResult, State};
use crate::style::template_style;

fn build_style(size: u16) -> ProgressStyle {
    template_style(
        size,
        true,
        |_| "🔧 {wide_msg} {pos:>5}/{len:<6}",
        |size| format!("[{{bar:{size}}}]"),
    )
}

pub struct DebugHandler {
    progress: ProgressBar,
    nb_lines: u64,
}

impl DebugHandler {
    pub fn new(state: &mut State) -> Self {
        let progress = ProgressBar::new_spinner()
            .with_style(build_style(state.term_size))
            .with_finish(ProgressFinish::Abandon);

        let progress = state.add(progress);

        DebugHandler {
            progress,
            nb_lines: 0,
        }
    }
}

impl Handler for DebugHandler {
    fn handle(&mut self, state: &mut State, _action: &Action) -> HandlerResult {
        let handlers_len = state.handlers_len as _;
        self.nb_lines += 1;

        self.progress
            .set_length(max(handlers_len, self.progress.length().unwrap_or(0)));

        self.progress
            .set_message(format!("Parsed {} lines of log", HumanCount(self.nb_lines)));

        self.progress.set_position(handlers_len);
        HandlerResult::Continue
    }

    fn resize(&mut self, _state: &mut State, size: u16) {
        self.progress.set_style(build_style(size))
    }
}
