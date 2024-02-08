use std::rc::Rc;
use std::time::Instant;

use console::style;
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};

use crate::action::{Action, ActionType, BuildStepId, ResultFields, StartFields};
use crate::handlers::logs::{LogHandler, LogsWindow};
use crate::state::{Handler, HandlerResult, State};
use crate::style::{
    format_build_target, format_short_build_target, MultiBar, PROGRESS_STYLE, PROGRESS_WIDTH,
    SPINNER_FREQ, SPINNER_STR,
};

pub fn builds_progress_style(done: u64, expected: u64, running: u64) -> ProgressStyle {
    let adv1 = (PROGRESS_WIDTH * done + expected / 2)
        .checked_div(expected)
        .unwrap_or(0);

    let adv2 = (PROGRESS_WIDTH * (done + running) + expected / 2)
        .checked_div(expected)
        .unwrap_or(0);

    let c_pos = "#";
    let c_run = style("-").green().bright().to_string();
    let p = "{spinner} {prefix} {wide_msg} {pos:>5}/{len:<6}";

    let bar = MultiBar([
        (c_pos, adv1),
        (&c_run, adv2 - adv1),
        (" ", PROGRESS_WIDTH - adv2),
    ]);

    ProgressStyle::with_template(&format!("{p} [{bar}] {{elapsed:>4}}"))
        .unwrap()
        .tick_strings(&SPINNER_STR)
}

pub fn handle_new_builds(state: &mut State, action: &Action) -> HandlerResult {
    if let Action::Start {
        action_type: ActionType::Builds,
        id,
        ..
    } = action
    {
        let progress = ProgressBar::new_spinner()
            .with_style(PROGRESS_STYLE.clone())
            .with_prefix("Build");

        let progress = state
            .multi_progress
            .insert_after(&state.separator, progress);

        let logs_window = Rc::new(LogsWindow::new(state, &progress, 5));
        progress.enable_steady_tick(SPINNER_FREQ);

        state.plug(Builds {
            id: *id,
            progress,
            builds_formatted: IndexMap::new(),
            logs_window,
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
                self.progress
                    .set_style(builds_progress_style(*done, *expected, *running));

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
}
