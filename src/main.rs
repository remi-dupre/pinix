pub mod action;
pub mod event;
pub mod handlers;
pub mod state;
pub mod style;

use self::handlers::builds::handle_new_builds;
use self::handlers::debug::DebugHandler;
use self::handlers::download::handle_new_download;
use self::handlers::message::handle_new_message;
use self::state::State;

fn main() {
    let mut state = State::default();
    let handle_debug = DebugHandler::new(&state);
    state.plug(handle_debug);
    state.plug(handle_new_builds);
    state.plug(handle_new_download);
    state.plug(handle_new_message);

    for line in std::io::stdin().lines() {
        let line = line.expect("could not read line");

        if let Some(action_str) = line.strip_prefix("@nix ") {
            let action = serde_json::from_str(action_str).expect("invalid JSON in action");
            state.handle(&action);
        } else {
            state.println(line);
            continue;
        };
    }

    // std::thread::sleep(std::time::Duration::from_millis(100));
}
