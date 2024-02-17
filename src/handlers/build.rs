use std::time::Instant;

use console::style;

use crate::action::{Action, BuildStepId, StartFields};
use crate::state::{Handler, HandlerResult, State};
use crate::style::format_build_target;

pub fn handle_new_build(state: &mut State, action: &Action) -> anyhow::Result<HandlerResult> {
    if let Action::Start {
        start_type: StartFields::Build { target, .. },
        id,
        ..
    } = action
    {
        state.plug(Build::new(*id, target.to_string()));
    }

    Ok(HandlerResult::Continue)
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
    fn on_action(&mut self, state: &mut State, action: &Action) -> anyhow::Result<HandlerResult> {
        match action {
            Action::Stop { id } if *id == self.id => {
                let icon = style("✓").green();
                let detail = style(format!("({:.0?})", self.start.elapsed())).dim();

                state.println(format!(
                    "{icon} Built {} {detail}",
                    format_build_target(&self.target)
                ))?;

                return Ok(HandlerResult::Close);
            }

            _ => {}
        }

        Ok(HandlerResult::Continue)
    }
}
