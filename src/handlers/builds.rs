use std::time::Instant;

use console::style;
use indexmap::IndexMap;
use indicatif::ProgressBar;

use crate::action::{Action, ActionType, BuildStepId, ResultFields, StartFields};
use crate::state::{Handler, HandlerResult, NewState};
use crate::style::{format_build_target, format_short_build_target, PROGRESS_STYLE, SPINNER_FREQ};

use super::logs::LogHandler;

pub fn handle_new_builds(state: &mut NewState, action: &Action) -> HandlerResult {
    if let Action::Start {
        action_type: ActionType::Builds,
        id,
        ..
    } = action
    {
        let progress = ProgressBar::new_spinner()
            .with_style(PROGRESS_STYLE.clone())
            .with_prefix("Build");

        let progress = state.multi_progress.insert(0, progress);
        progress.enable_steady_tick(SPINNER_FREQ);
        state.plug(Builds {
            id: *id,
            progress,
            builds_formatted: IndexMap::new(),
        });
    }

    HandlerResult::Continue
}

/// Keep track of current group of builds
struct Builds {
    id: BuildStepId,
    progress: ProgressBar,
    builds_formatted: IndexMap<BuildStepId, String>,
}

impl Builds {
    fn update_message(&self) {
        let all_builds: Vec<_> = self.builds_formatted.values().map(String::as_str).collect();
        self.progress.set_message(all_builds.join(", "));
    }
}

impl Handler for Builds {
    fn handle(&mut self, state: &mut NewState, action: &Action) -> HandlerResult {
        match action {
            // New build
            Action::Start {
                action_type: ActionType::Build,
                id,
                fields: StartFields::Build((target, _, _, _)),
                ..
            } => {
                self.builds_formatted
                    .insert(*id, format_short_build_target(target));

                self.update_message();
                state.plug(Build::new(*id, target.to_string()));
                state.plug(LogHandler::new(*id));
            }

            // Stop build
            Action::Stop { id } if self.builds_formatted.contains_key(id) => {
                self.builds_formatted.shift_remove(id);
                self.update_message();
            }

            // Update progress of builds
            Action::Result {
                action_type: ActionType::Build,
                id,
                fields: ResultFields::Progress([done, expected, ..]),
            } if *id == self.id => {
                self.progress.set_length(*expected);
                self.progress.set_position(*done);
            }

            // Stop builds
            Action::Stop { id } if *id == self.id => {
                if let Some(nb_built) = self.progress.length() {
                    let icon = style("⯈").green();
                    let detail = style(format!("({:.0?})", self.progress.duration())).dim();
                    state.println(format!("{icon} Built {nb_built} derivations {detail}"));
                }

                self.progress.finish_and_clear();
                return HandlerResult::Close;
            }

            _ => {}
        }

        HandlerResult::Continue
    }
}

/// Keep track of a single build
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
    fn handle(&mut self, state: &mut NewState, action: &Action) -> HandlerResult {
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
}
