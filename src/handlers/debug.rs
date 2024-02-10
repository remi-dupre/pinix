use std::cmp::max;

use indicatif::{HumanCount, ProgressBar, ProgressFinish, ProgressStyle};
use once_cell::sync::Lazy;

use crate::action::Action;
use crate::state::{Handler, HandlerResult, State};
use crate::style::PROGRESS_WIDTH;

pub static STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    let p = "🔧 {wide_msg} {pos:>5}/{len:<6}";

    ProgressStyle::with_template(&format!("{p} [{{bar:{PROGRESS_WIDTH}}}] {{elapsed:>4}}"))
        .unwrap()
        .progress_chars("## ")
});

pub struct DebugHandler {
    progress: ProgressBar,
    nb_lines: u64,
}

impl DebugHandler {
    pub fn new(state: &mut State) -> Self {
        let progress = ProgressBar::new_spinner()
            .with_style(STYLE.clone())
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
}
