use std::rc::Rc;
use std::time::Instant;

use anyhow::Context;
use console::style;
use futures::TryStreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::action::Action;
use crate::handlers::build::handle_new_build;
use crate::handlers::builds_group::handle_new_builds_group;
use crate::handlers::debug::DebugHandler;
use crate::handlers::download::handle_new_download;
use crate::handlers::downloads_group::handle_new_downloads_group;
use crate::handlers::message::handle_new_message;
use crate::handlers::unknown::handle_new_unknown;
use crate::wrapper::command::NixCommand;
use crate::wrapper::stream::{OutputStream, StreamedPipes};

#[derive(Eq, PartialEq)]
pub enum HandlerResult {
    Continue,
    Close,
}

pub trait Handler {
    fn on_action(&mut self, state: &mut State, action: &Action) -> HandlerResult;
    fn on_resize(&mut self, _state: &mut State);
}

impl<F: FnMut(&mut State, &Action) -> HandlerResult> Handler for F {
    fn on_action(&mut self, state: &mut State, action: &Action) -> HandlerResult {
        self(state, action)
    }

    fn on_resize(&mut self, _state: &mut State) {}
}

pub struct State<'s> {
    pub cmd: &'s NixCommand,
    pub multi_progress: Rc<MultiProgress>,
    pub handlers: Vec<Box<dyn Handler + 's>>,
    pub term_size: u16,

    // First displayed line, only appears when other lines do
    separator: Option<ProgressBar>,

    /// Keep track of the handler could while applying them. Usefull for
    /// debugging.
    pub handlers_len: usize,
}

impl<'s> State<'s> {
    pub fn new(cmd: &'s NixCommand) -> Self {
        let (_, term_size) = console::Term::stderr().size();
        let multi_progress = Rc::new(MultiProgress::default());

        let mut state = Self {
            cmd,
            multi_progress,
            handlers: Vec::new(),
            term_size,
            separator: None,
            handlers_len: 0,
        };

        if cmd.args.debug {
            let debug_bar = DebugHandler::new(&mut state);
            state.plug(debug_bar);
        }

        state.plug(handle_new_build);
        state.plug(handle_new_builds_group);
        state.plug(handle_new_download);
        state.plug(handle_new_downloads_group);
        state.plug(handle_new_message);
        state.plug(handle_new_unknown);
        state
    }
}

impl<'s> State<'s> {
    pub fn handle(&mut self, action: &Action) {
        // Move out handlers to allow borrowing self
        let mut prev_handlers = std::mem::take(&mut self.handlers);

        // Check if terminal was resized
        let (_, term_size) = console::Term::stderr().size();

        if term_size != self.term_size {
            self.term_size = term_size;

            for handler in &mut prev_handlers {
                handler.on_resize(self)
            }
        }

        if let Some(separator) = &self.separator {
            separator.tick();
        }

        // Applies handles
        prev_handlers.retain_mut(|h| h.on_action(self, action) == HandlerResult::Continue);

        // Put back remaining handlers
        let mut new_handlers = std::mem::replace(&mut self.handlers, prev_handlers);
        self.handlers.append(&mut new_handlers);
        self.handlers_len = self.handlers.len();
    }

    pub fn plug<H: Handler + 's>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler) as _)
    }

    pub fn add(&mut self, pb: ProgressBar) -> ProgressBar {
        let separator = self.separator.get_or_insert_with(|| {
            let separator = ProgressBar::new_spinner()
                .with_style(
                    ProgressStyle::default_spinner()
                        .template(&style("·· {prefix} {wide_msg:<}").dim().to_string())
                        .expect("invalid template"),
                )
                .with_prefix(format!("Running {}", self.cmd.program.as_str()))
                .with_message("·".repeat(512))
                .with_finish(ProgressFinish::AndClear);

            let separator = self.multi_progress.insert(0, separator);
            separator.set_length(0);
            separator
        });

        self.multi_progress.insert_after(separator, pb)
    }

    pub fn remove_separator(&mut self) {
        self.separator.take();
    }

    pub fn println(&self, msg: impl AsRef<str>) {
        self.multi_progress
            .println(msg)
            .expect("could not print line")
    }
}

pub async fn monitor_logs(
    cmd: &NixCommand,
    log_stream: impl StreamedPipes<'_>,
) -> anyhow::Result<()> {
    let mut state = State::new(cmd);
    let start_time = Instant::now();

    let mut record_file = {
        if let Some(path) = &cmd.args.record {
            let file = tokio::fs::File::create(&path)
                .await
                .context("could not open record file")?;

            Some(BufWriter::new(file))
        } else {
            None
        }
    };

    log_stream
        .try_fold(&mut record_file, move |mut record_file, (output, line)| {
            let elapsed = start_time.elapsed();

            match output {
                OutputStream::StdOut => state.println(&line),
                OutputStream::StdErr => {
                    if let Some(action_str) = line.strip_prefix("@nix ") {
                        let action =
                            serde_json::from_str(action_str).expect("invalid JSON in action");
                        state.handle(&action);
                    } else {
                        state.println(&line);
                    }
                }
            };

            async move {
                if let Some(file) = &mut record_file {
                    let line = format!("{} {:07} {line}\n", output.as_str(), elapsed.as_millis());

                    file.write_all(line.as_bytes())
                        .await
                        .context("error writing record file")?;
                }

                Ok(record_file)
            }
        })
        .await?;

    if let Some(mut file) = record_file {
        file.flush().await.context("error saving record file")?;
    }

    Ok(())
}
