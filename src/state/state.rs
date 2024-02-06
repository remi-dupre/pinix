use std::collections::HashMap;
use std::rc::Rc;

use indicatif::MultiProgress;

use crate::action::BuildStepId;
use crate::event::EventRegister;
use crate::state::build::StateBuild;
use crate::state::download::StateDownload;

#[derive(Default)]
pub struct State {
    pub main_progress: Rc<MultiProgress>,
    pub state_builds: HashMap<BuildStepId, StateBuild>,
    pub state_download: HashMap<BuildStepId, StateDownload>,
    pub file_transfers: HashMap<BuildStepId, BuildStepId>,
    pub msg_buffer: HashMap<BuildStepId, Vec<String>>,

    pub events_builds_start: EventRegister<(), BuildStepId>,
    pub events_build_start: EventRegister<(), (BuildStepId, String)>,
    pub events_download_start: EventRegister<(), (BuildStepId, String)>,
    pub events_message: EventRegister<(), str>,
    pub events_stop: EventRegister<BuildStepId, ()>,
    pub events_transfer_start: EventRegister<BuildStepId, BuildStepId>,
    pub events_status: EventRegister<BuildStepId, [u64; 4]>,
}

impl State {
    // pub fn message(&self, msg: &str) -> anyhow::Result<()> {
    //     self.main_progress
    //         .println(msg)
    //         .context("couldn't write line")
    // }
    //
    // pub fn message_for(&mut self, id: BuildStepId, msg: String) {
    //     self.msg_buffer.entry(id).or_default().push(msg);
    // }
    //
    // pub fn stop(&mut self, id: BuildStepId) {
    //     let mut has_stopped = false;
    //
    //     if let Some(state_builds) = self.state_builds.get(&id) {
    //         state_builds.stop();
    //         has_stopped = true;
    //     }
    //
    //     if let Some(state_download) = self.state_download.get(&id) {
    //         state_download.stop();
    //         has_stopped = true;
    //     }
    //
    //     for state_builds in self.state_builds.values_mut() {
    //         has_stopped |= state_builds.build_stop(id);
    //     }
    //
    //     if let Some(messages) = self.msg_buffer.remove(&id) {
    //         let messages_len = messages.len();
    //
    //         for (i, msg) in messages.into_iter().enumerate() {
    //             let prefix = {
    //                 if !has_stopped && messages_len == 1 {
    //                     '-'
    //                 } else if !has_stopped && i == 0 {
    //                     '┌'
    //                 } else if i + 1 == messages_len {
    //                     '└'
    //                 } else {
    //                     '│'
    //                 }
    //             };
    //
    //             self.main_progress
    //                 .println(style(format!("{prefix} {msg}")).dim().to_string())
    //                 .expect("couldn't write line");
    //         }
    //     }
    // }
    //
    // pub fn download_new(&mut self, id: BuildStepId, path: &str) {
    //     self.state_download
    //         .insert(id, StateDownload::new(self.main_progress.clone(), path));
    // }
    //
    // pub fn download_transfer(&mut self, download_id: BuildStepId, transfer_id: BuildStepId) {
    //     self.file_transfers.insert(download_id, transfer_id);
    // }
    //
    // pub fn builds_new(&mut self, id: BuildStepId) {
    //     self.state_builds
    //         .insert(id, StateBuild::new(self.main_progress.clone()));
    // }
    //
    // pub fn build_new(&mut self, id: BuildStepId, msg: &str) {
    //     for builds in self.state_builds.values_mut().filter(|b| b.is_active()) {
    //         builds.build_new(id, msg);
    //     }
    // }
    //
    // pub fn update_build(&mut self, id: BuildStepId, done: u64, expected: u64) {
    //     if let Some(state_builds) = self.state_builds.get(&id) {
    //         state_builds.update(done, expected)
    //     }
    //
    //     if let Some(transfers_to) = self.file_transfers.get(&id) {
    //         if let Some(state_download) = self.state_download.get(transfers_to) {
    //             state_download.update(done, expected)
    //         }
    //     }
    // }
}
