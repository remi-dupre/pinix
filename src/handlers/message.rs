use crate::action::Action;
use crate::state::{HandlerResult, NewState};

pub fn handle_new_message(state: &mut NewState, action: &Action) -> HandlerResult {
    if let Action::Msg { msg, .. } = action {
        state.println(msg)
    }

    HandlerResult::Continue
}
