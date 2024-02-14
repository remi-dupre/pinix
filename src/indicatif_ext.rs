use std::time::Duration;

use indicatif::ProgressBar;
use tokio::time::MissedTickBehavior;

/// Extension trait for indicatif's ProgressBar
pub trait ProgressBarExt {
    fn get_bar(&self) -> &ProgressBar;

    fn spawn_steady_tick(&self, interval: Duration) {
        let pb_weak = self.get_bar().downgrade();
        let mut interval = tokio::time::interval(interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                let Some(pb) = pb_weak.upgrade() else { break };
                pb.tick();
            }
        });
    }
}

impl ProgressBarExt for ProgressBar {
    fn get_bar(&self) -> &ProgressBar {
        self
    }
}
