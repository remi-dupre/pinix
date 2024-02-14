use std::time::Instant;

use console::style;

use crate::action::{Action, ActionType, BuildStepId, StartFields};
use crate::state::{Handler, HandlerResult, State};
use crate::style::format_build_target;

pub fn handle_new_build(state: &mut State, action: &Action) -> HandlerResult {
    if let Action::Start {
        action_type: ActionType::Build,
        id,
        fields: StartFields::Build((target, _, _, _)),
        ..
    } = action
    {
        state.plug(Build::new(*id, target.to_string()));
    }

    HandlerResult::Continue
}

struct Build {
    id: BuildStepId,
    target: String,
    start: Instant,
}

impl Build {
    fn new(id: BuildStepId, target: String) -> Self {
        Self {
            id,
            target,
            start: Instant::now(),
        }
    }
}

impl Handler for Build {
    fn on_action(&mut self, state: &mut State, action: &Action) -> HandlerResult {
        match action {
            Action::Stop { id } if *id == self.id => {
                let icon = style("✓").green();
                let detail = style(format!("({:.0?})", self.start.elapsed())).dim();

                state.println(format!(
                    "{icon} Built {} {detail}",
                    format_build_target(&self.target)
                ));

                return HandlerResult::Close;
            }

            _ => {}
        }

        HandlerResult::Continue
    }

    fn on_resize(&mut self, _state: &mut State) {}
}
