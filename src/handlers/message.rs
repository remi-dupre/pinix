use crate::action::Action;
use crate::state::{HandlerResult, State};

pub fn handle_new_message(state: &mut State, action: &Action) -> anyhow::Result<HandlerResult> {
    if let Action::Msg { msg, .. } = action {
        state.println(msg)?;
    }

    Ok(HandlerResult::Continue)
}
