use std::rc::Rc;

use console::style;
use indicatif::ProgressBar;

use crate::action::{Action, BuildStepId, ResultFields};
use crate::state::{Handler, HandlerResult, State};
use crate::style::template_style;

#[derive(Default)]
pub struct LogHandler {
    id: BuildStepId,
    logs: Vec<String>,
    logs_window: Option<Rc<LogsWindow>>,
}

impl LogHandler {
    pub fn new(id: BuildStepId) -> Self {
        Self {
            id,
            logs: Vec::new(),
            logs_window: None,
        }
    }

    pub fn with_logs_window(mut self, logs_window: Rc<LogsWindow>) -> Self {
        self.logs_window = Some(logs_window);
        self
    }
}

impl Handler for LogHandler {
    fn handle(&mut self, state: &mut State, action: &Action) -> HandlerResult {
        match action {
            Action::Result {
                id,
                fields: ResultFields::Msg([msg]),
                ..
            } if *id == self.id => {
                self.logs.push(msg.to_string());

                if let Some(logs_window) = &self.logs_window {
                    logs_window.log(msg.to_string());
                }
            }

            Action::Stop { id } if *id == self.id => {
                let logs_len = self.logs.len();

                for (i, line) in self.logs.iter().enumerate() {
                    let prefix = if i + 1 == logs_len { '└' } else { '│' };
                    state.println(style(format!("{prefix} {line}")).dim().to_string());
                }

                return HandlerResult::Close;
            }

            _ => {}
        }

        HandlerResult::Continue
    }

    fn resize(&mut self, _state: &mut State, _size: u16) {}
}

pub struct LogsWindow {
    log_lines: Vec<ProgressBar>,
}

impl LogsWindow {
    pub fn new(state: &mut State, after: &ProgressBar, nb_lines: usize) -> Self {
        let mut log_lines = Vec::new();

        for i in 0..nb_lines {
            let prefix = if i + 1 == nb_lines { "└" } else { "│" };
            let prev = log_lines.last().unwrap_or(after);

            let next = state.multi_progress.insert_after(
                prev,
                ProgressBar::new_spinner()
                    .with_style(template_style(
                        state.term_size,
                        false,
                        |_| style("{prefix} {wide_msg}").dim(),
                        |_| "",
                    ))
                    .with_prefix(prefix),
            );

            log_lines.push(next);
        }

        Self { log_lines }
    }

    pub fn log(&self, msg: String) {
        for (prev, next) in self.log_lines.iter().zip(&self.log_lines[1..]) {
            if !next.message().is_empty() {
                prev.set_message(next.message());
            }
        }

        if let Some(last_line) = self.log_lines.last() {
            last_line.set_message(msg);
        }
    }

    pub fn resize(&self, _size: u16) {
        for line in &self.log_lines {
            line.tick();
        }
    }
}
