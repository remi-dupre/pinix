use std::rc::Rc;
use std::time::Instant;

use console::style;
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};

use crate::action::{Action, ActionType, BuildStepId, ResultFields, StartFields};
use crate::handlers::logs::{LogHandler, LogsWindow};
use crate::state::{Handler, HandlerResult, State};
use crate::style::{format_build_target, format_short_build_target, template_style, MultiBar};

pub fn builds_progress_style(size: u16, done: u64, expected: u64, running: u64) -> ProgressStyle {
    template_style(
        size,
        true,
        |size| match size {
            0..=50 => "{prefix} {wide_msg}",
            _ => "{prefix} {wide_msg} {pos:>5}/{len:<6}",
        },
        |size| {
            let size = u64::from(size);

            let adv1 = (size * done + expected / 2)
                .checked_div(expected)
                .unwrap_or(0);

            let adv2 = (size * (done + running) + expected / 2)
                .checked_div(expected)
                .unwrap_or(0);

            let c_pos = "#";
            let c_run = style("-").blue().bright().to_string();
            let bar = MultiBar([(c_pos, adv1), (&c_run, adv2 - adv1), (" ", size - adv2)]);
            format!("[{bar}]")
        },
    )
}

pub fn handle_new_builds(state: &mut State, action: &Action) -> HandlerResult {
    if let Action::Start {
        action_type: ActionType::Builds,
        id,
        ..
    } = action
    {
        let progress = ProgressBar::new_spinner()
            .with_style(builds_progress_style(state.term_size, 0, 0, 0))
            .with_prefix("Build");

        let progress = state.add(progress);
        let logs_window = Rc::new(LogsWindow::new(state, &progress, 5));

        state.plug(Builds {
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
struct Builds {
    id: BuildStepId,
    progress: ProgressBar,
    builds_formatted: IndexMap<BuildStepId, String>,
    logs_window: Rc<LogsWindow>,
    last_state: [u64; 3],
}

impl Builds {
    fn update_message(&self) {
        let all_builds: Vec<_> = self.builds_formatted.values().map(String::as_str).collect();
        self.progress.set_message(all_builds.join(", "));
    }
}

impl Handler for Builds {
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
                state.plug(Build::new(*id, target.to_string()));
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

                self.progress.set_style(builds_progress_style(
                    state.term_size,
                    *done,
                    *expected,
                    *running,
                ));

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
        let [done, expected, running] = self.last_state;

        self.progress
            .set_style(builds_progress_style(size, done, expected, running));
        self.progress.tick();
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
    fn handle(&mut self, state: &mut State, action: &Action) -> HandlerResult {
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

    fn resize(&mut self, _state: &mut State, _size: u16) {}
}
