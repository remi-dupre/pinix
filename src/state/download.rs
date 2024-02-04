use std::rc::Rc;

use console::style;
use indicatif::{HumanBytes, MultiProgress, ProgressBar};

use crate::style::{format_build_target, DOWNLOAD_STYLE};

pub struct StateDownload {
    main_progress: Rc<MultiProgress>,
    progress: ProgressBar,
}

impl StateDownload {
    pub fn new(main_progress: Rc<MultiProgress>, path: &str) -> Self {
        let progress = ProgressBar::new_spinner()
            .with_style(DOWNLOAD_STYLE.clone())
            .with_prefix("Download")
            .with_message(format_build_target(path));

        // progress.enable_steady_tick(SPINNER_FREQ);

        Self {
            progress: main_progress.insert(0, progress),
            main_progress,
        }
    }

    pub fn update(&self, done: u64, expected: u64) {
        self.progress.set_length(expected);
        self.progress.set_position(done);
    }

    pub fn stop(&self) {
        let msg_main = format!(
            "{} Downloaded {}",
            style("✓").green(),
            self.progress.message(),
        );

        let msg_stats = style(format!(
            " ({}, {:.0?})",
            HumanBytes(self.progress.position()),
            self.progress.duration(),
        ))
        .dim()
        .to_string();

        self.main_progress
            .println(msg_main + &msg_stats)
            .expect("couldn't print line");

        self.progress.finish_and_clear();
    }
}
