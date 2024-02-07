use console::style;

use crate::action::{Action, BuildStepId, ResultFields};
use crate::state::{Handler, HandlerResult, State};

#[derive(Default)]
pub struct LogHandler {
    id: BuildStepId,
    logs: Vec<String>,
}

impl LogHandler {
    pub fn new(id: BuildStepId) -> Self {
        Self {
            id,
            logs: Vec::new(),
        }
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
}
