use std::collections::HashMap;
use std::rc::Rc;

use console::style;
use indicatif::{MultiProgress, ProgressBar};

use crate::action::BuildStepId;
use crate::event::EventTriggerResult;
use crate::style::{format_build_target, format_short_build_target, PROGRESS_STYLE, SPINNER_FREQ};

use super::state::State;

#[derive(Default)]
pub struct StateBuilds {
    children: HashMap<BuildStepId, String>,
    progress: ProgressBar,
}

impl StateBuilds {
    pub fn connect(state: Rc<State>) {
        state.events_builds_start.connect((), {
            let state = state.clone();

            move |id| {
                Self::init_builds(state);
                EventTriggerResult::Continue
            }
        });
    }

    fn init_builds(state: Rc<State>) {
        let mut builds = StateBuilds::default();
    }

    // pub fn new(main_progress: Rc<MultiProgress>) -> Self {
    //     let progress = ProgressBar::new_spinner()
    //         .with_style(PROGRESS_STYLE.clone())
    //         .with_prefix("Build");
    //
    //     progress.enable_steady_tick(SPINNER_FREQ);
    //
    //     Self {
    //         children: HashMap::new(),
    //         progress: main_progress.insert(0, progress),
    //         main_progress,
    //     }
    // }
    //
    // pub fn is_active(&self) -> bool {
    //     !self.progress.is_finished()
    // }
    //
    // pub fn update(&self, done: u64, expected: u64) {
    //     self.progress.set_length(expected);
    //     self.progress.set_position(done);
    // }
    //
    // pub fn stop(&self) {
    //     if self.progress.length().unwrap_or(0) == 0 {
    //         self.progress.finish_and_clear();
    //     } else {
    //         self.progress.finish_with_message("finished")
    //     }
    // }
    //
    // fn rebuild_message(&self) {
    //     let mut children_str: Vec<_> = self
    //         .children
    //         .values()
    //         .map(|s| format_short_build_target(s))
    //         .collect();
    //
    //     children_str.sort();
    //     self.progress.set_message(children_str.join(", "));
    // }
    //
    // pub fn build_new(&mut self, id: BuildStepId, msg: &str) {
    //     self.children.insert(id, msg.to_string());
    //     self.rebuild_message();
    // }
    //
    // pub fn build_stop(&mut self, id: BuildStepId) -> bool {
    //     let Some(built_path) = self.children.remove(&id) else {
    //         return false;
    //     };
    //
    //     self.rebuild_message();
    //
    //     self.main_progress
    //         .println(format!(
    //             "{} Built {}",
    //             style("✓").green(),
    //             format_build_target(&built_path)
    //         ))
    //         .expect("couldn't print line");
    //
    //     true
    // }
}
