pub mod action;
pub mod event;
pub mod state;
pub mod style;

use std::rc::Rc;
use std::time::Duration;

use anyhow::{bail, Context};

use crate::action::StartFields;
use crate::event::EventTriggerResult;

use self::action::{Action, ActionType, ResultFields};
use self::state::build::StateBuilds;
use self::state::download::StateDownload;
use self::state::state::State;

fn main() -> anyhow::Result<()> {
    let mut state = Rc::new(State::default());

    state.events_message.connect((), {
        let state = state.clone();
        move |msg| {
            state
                .main_progress
                .println(msg)
                .expect("could not print line");

            EventTriggerResult::Continue
        }
    });

    StateDownload::connect(state.clone());
    StateBuilds::connect(state.clone());

    for line in std::io::stdin().lines() {
        let line = line?;

        let Some(action_str) = line.strip_prefix("@nix ") else {
            state.events_message.trigger(&(), &line);
            continue;
        };

        match serde_json::from_str::<Action>(action_str).context("invalid JSON in action")? {
            Action::Msg { level: _, msg } => state.events_message.trigger(&(), &msg),

            Action::Start {
                action_type,
                id,
                level: _,
                parent,
                text: _,
                fields,
            } => match action_type {
                ActionType::Builds => state.events_builds_start.trigger(&(), &id),
                ActionType::Build => {
                    let StartFields::Build((name, _, _, _)) = fields else {
                        bail!("Invalid fields for build: {fields:?}");
                    };

                    state.events_build_start.trigger(&(), &(id, name))
                }
                ActionType::CopyPath => {
                    let StartFields::Copy([path, _, _]) = fields else {
                        bail!("Invalid fields for copy: {fields:?}");
                    };

                    state.events_download_start.trigger(&(), &(id, path));
                }
                ActionType::FileTransfer => state.events_transfer_start.trigger(&parent, &id),

                _ => {}
            },

            Action::Result {
                action_type,
                id,
                fields,
            } => match (action_type, fields) {
                (ActionType::Build, ResultFields::Progress(status)) => {
                    state.events_status.trigger(&id, &status)
                }
                (_, ResultFields::Msg([msg])) => {
                    // state.message_for(id, msg)
                }
                _ => {}
            },

            Action::Stop { id } => {
                state.events_stop.trigger(&id, &());
            }
        }

        std::thread::sleep(Duration::from_micros(20));
    }

    Ok(())
}
