use std::rc::Rc;

use console::style;
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;

use crate::action::{Action, ActionType, BuildStepId, ResultFields, StartFields};
use crate::handlers::logs::{LogHandler, LogsWindow};
use crate::state::{Handler, HandlerResult, State};
use crate::style::{format_short_build_target, template_style, MultiBar};

static C_RUN: Lazy<String> = Lazy::new(|| style("-").blue().bright().to_string());

pub fn get_style(size: u16) -> ProgressStyle {
    template_style(
        size,
        true,
        |size| match size {
            0..=50 => "Build {wide_msg}",
            _ => "Build {wide_msg} {pos:>5}/{len:<6}",
        },
        |_| "[{prefix}]",
    )
}

pub fn handle_new_builds_group(state: &mut State, action: &Action) -> HandlerResult {
    if let Action::Start {
        action_type: ActionType::Builds,
        id,
        ..
    } = action
    {
        let progress = ProgressBar::new_spinner().with_style(get_style(state.term_size));
        let progress = state.add(progress);
        let logs_window = Rc::new(LogsWindow::new(state, &progress, 5));

        state.plug(BuildGroup {
            id: *id,
            progress,
            builds_formatted: IndexMap::new(),
            logs_window,
            last_state: [0; 3],
        });
    }

    HandlerResult::Continue
}

/// Keep track of current group of builds
struct BuildGroup {
    id: BuildStepId,
    progress: ProgressBar,
    builds_formatted: IndexMap<BuildStepId, String>,
    logs_window: Rc<LogsWindow>,
    last_state: [u64; 3],
}

impl BuildGroup {
    fn update_message(&self) {
        let all_builds: Vec<_> = self.builds_formatted.values().map(String::as_str).collect();
        self.progress.set_message(all_builds.join(", "));
    }

    fn build_bar(&self, size: u16) -> MultiBar<'_, 3> {
        let size = u64::from(size / 3);
        let [done, expected, running] = self.last_state;

        let adv1 = (size * done + expected / 2)
            .checked_div(expected)
            .unwrap_or(0);

        let adv2 = (size * (done + running) + expected / 2)
            .checked_div(expected)
            .unwrap_or(0);

        let c_pos = "#";

        MultiBar([
            (c_pos, adv1),
            (C_RUN.as_str(), adv2 - adv1),
            (" ", size - adv2),
        ])
    }
}

impl Handler for BuildGroup {
    fn handle(&mut self, state: &mut State, action: &Action) -> HandlerResult {
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
                state.plug(LogHandler::new(*id).with_logs_window(self.logs_window.clone()));
            }

            // Stop build
            Action::Stop { id } if self.builds_formatted.shift_remove(id).is_some() => {
                self.update_message();
            }

            // Update progress of builds
            Action::Result {
                action_type: ActionType::Build,
                id,
                fields: ResultFields::Progress([done, expected, running, ..]),
            } if *id == self.id => {
                self.last_state = [*done, *expected, *running];

                self.progress
                    .set_prefix(self.build_bar(state.term_size).to_string());

                self.progress.set_length(*expected);
                self.progress.set_position(*done);
            }

            // Stop builds
            Action::Stop { id } if *id == self.id => {
                let nb_built = self.progress.length().unwrap_or(0);

                if nb_built > 0 {
                    let icon = style("⯈").green();
                    let detail = style(format!("({:.0?})", self.progress.duration())).dim();
                    state.println(format!("{icon} Built {nb_built} derivations {detail}"));
                }

                state.remove_separator();
                self.progress.finish_and_clear();
                return HandlerResult::Close;
            }

            _ => {}
        }

        HandlerResult::Continue
    }

    fn resize(&mut self, _state: &mut State, size: u16) {
        self.logs_window.resize(size);
        self.progress.set_style(get_style(size));
        self.progress.set_prefix(self.build_bar(size).to_string());
        self.progress.tick();
    }
}
