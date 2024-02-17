use std::rc::Rc;
use std::time::Instant;

use anyhow::Context;
use console::style;
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
use crate::wrapper::stream::{MergedStreams, OutputStream};

#[derive(Eq, PartialEq)]
pub enum HandlerResult {
    Continue,
    Close,
}

pub trait Handler {
    fn on_action<'a>(
        &mut self,
        state: &mut State,
        action: &'a Action<'a>,
    ) -> anyhow::Result<HandlerResult>;

    fn on_resize(&mut self, _state: &mut State) -> anyhow::Result<()> {
        Ok(())
    }
}

impl<F: FnMut(&mut State, &Action) -> anyhow::Result<HandlerResult>> Handler for F {
    fn on_action<'a>(
        &mut self,
        state: &mut State,
        action: &'a Action<'a>,
    ) -> anyhow::Result<HandlerResult> {
        self(state, action)
    }
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
    pub fn handle(&mut self, action: &Action) -> anyhow::Result<()> {
        // Move out handlers to allow borrowing self
        let mut prev_handlers = std::mem::take(&mut self.handlers);

        // Check if terminal was resized
        let (_, term_size) = console::Term::stderr().size();

        if term_size != self.term_size {
            self.term_size = term_size;

            for handler in &mut prev_handlers {
                handler.on_resize(self)?;
            }
        }

        if let Some(separator) = &self.separator {
            separator.tick();
        }

        // Applies handles
        let mut retain_result = Ok(());

        prev_handlers.retain_mut(|h| match h.on_action(self, action) {
            Ok(x) => x == HandlerResult::Continue,
            Err(err) => {
                retain_result = Err(err);
                false
            }
        });

        // Put back remaining handlers
        let mut new_handlers = std::mem::replace(&mut self.handlers, prev_handlers);
        self.handlers.append(&mut new_handlers);
        self.handlers_len = self.handlers.len();
        retain_result
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

    pub fn println(&self, msg: impl AsRef<str>) -> anyhow::Result<()> {
        self.multi_progress
            .println(msg)
            .context("Could not print line")
    }
}

pub async fn monitor_logs<'c>(
    cmd: &NixCommand,
    mut log_stream: MergedStreams<'c>,
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

    while let Some((output, line)) = log_stream.next_line().await? {
        let line = std::str::from_utf8(line).context("invalid utf-8")?;

        if let Some(file) = &mut record_file {
            let elapsed = start_time.elapsed();
            let line = format!("{} {:07} {line}\n", output.as_str(), elapsed.as_millis());

            file.write_all(line.as_bytes())
                .await
                .context("error writing record file")?;
        }

        match output {
            OutputStream::StdOut => {
                state.println(line)?;
            }
            OutputStream::StdErr => {
                if let Some(action_raw) = line.strip_prefix("@nix ") {
                    let action = Action::parse(action_raw)?;
                    state.handle(&action)?;
                } else {
                    state.println(line)?
                }
            }
        }
    }

    if let Some(mut file) = record_file {
        file.flush().await.context("error saving record file")?;
    }

    Ok(())
}
