use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use once_cell::sync::Lazy;

use crate::action::{Action, ActionType, BuildStepId};
use crate::handlers::logs::LogHandler;
use crate::state::{Handler, HandlerResult, State};
use crate::style::{SPINNER_FREQ, SPINNER_STR};

static STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template("{spinner} {wide_msg} {elapsed}")
        .unwrap()
        .progress_chars("## ")
        .tick_strings(&SPINNER_STR)
});

pub fn handle_new_unknown(state: &mut State, action: &Action) -> HandlerResult {
    if let Action::Start {
        action_type: ActionType::Unknown,
        id,
        text,
        ..
    } = action
    {
        let handler = Unknown::new(*id, text.as_str(), state);
        state.plug(handler);
        state.plug(LogHandler::new(*id));
    };

    HandlerResult::Continue
}

struct Unknown {
    id: BuildStepId,
    _progress: ProgressBar,
}

impl Unknown {
    fn new(id: BuildStepId, text: &str, state: &mut State) -> Self {
        let first_char = text.chars().next().unwrap_or(' ').to_ascii_uppercase();
        let first_char_len = text.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        let message = format!("{first_char}{}", &text[first_char_len..]);

        let progress = ProgressBar::new_spinner()
            .with_style(STYLE.clone())
            .with_message(message)
            .with_finish(ProgressFinish::AndClear);

        let progress = state.add(progress);
        progress.enable_steady_tick(SPINNER_FREQ);

        Self {
            id,
            _progress: progress,
        }
    }
}

impl Handler for Unknown {
    fn handle(&mut self, _state: &mut State, action: &Action) -> HandlerResult {
        if matches!(action , Action::Stop { id, .. } if *id == self.id) {
            HandlerResult::Close
        } else {
            HandlerResult::Continue
        }
    }
}
