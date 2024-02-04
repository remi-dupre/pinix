pub mod action;
pub mod state;
pub mod style;

use std::time::Duration;

use anyhow::{bail, Context};

use crate::action::StartFields;

use self::action::{Action, ActionType, ResultFields};
use self::state::state::State;

fn main() -> anyhow::Result<()> {
    let mut state = State::default();

    for line in std::io::stdin().lines() {
        let line = line?;

        let Some(action_str) = line.strip_prefix("@nix ") else {
            state.message(&line)?;
            continue;
        };

        match serde_json::from_str::<Action>(action_str).context("invalid JSON in action")? {
            Action::Msg { level: _, msg } => state.message(&msg)?,

            Action::Start {
                action_type,
                id,
                level: _,
                parent,
                text: _,
                fields,
            } => match action_type {
                ActionType::Builds => state.builds_new(id),
                ActionType::Build => {
                    let StartFields::Build((name, _, _, _)) = fields else {
                        bail!("Invalid fields for build: {fields:?}");
                    };

                    state.build_new(id, &name)
                }
                ActionType::CopyPath => {
                    let StartFields::Copy([path, _, _]) = fields else {
                        bail!("Invalid fields for copy: {fields:?}");
                    };

                    state.download_new(id, &path);
                }
                ActionType::FileTransfer => {
                    state.download_transfer(id, parent);
                }
                _ => {}
            },

            Action::Result {
                action_type,
                id,
                fields,
            } => match (action_type, fields) {
                (ActionType::Build, ResultFields::Progress([done, expected, _, _])) => {
                    state.update_build(id, done, expected)
                }
                (_, ResultFields::Msg([msg])) => state.message_for(id, msg),
                _ => {}
            },

            Action::Stop { id } => state.stop(id),
        }

        std::thread::sleep(Duration::from_micros(20));
    }

    Ok(())
}
