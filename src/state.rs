use std::rc::Rc;

use console::style;
use futures::TryStreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::action::Action;
use crate::handlers::builds::handle_new_builds;
use crate::handlers::debug::DebugHandler;
use crate::handlers::download::handle_new_download;
use crate::handlers::message::handle_new_message;
use crate::wrapper::command::NixCommand;
use crate::wrapper::stream::{StreamedPipes, WrappedLine};

#[derive(Eq, PartialEq)]
pub enum HandlerResult {
    Continue,
    Close,
}

pub trait Handler {
    fn handle(&mut self, state: &mut State, action: &Action) -> HandlerResult;
}

impl<F: FnMut(&mut State, &Action) -> HandlerResult> Handler for F {
    fn handle(&mut self, state: &mut State, action: &Action) -> HandlerResult {
        self(state, action)
    }
}

pub struct State<'s> {
    pub cmd: &'s NixCommand,
    pub multi_progress: Rc<MultiProgress>,
    pub handlers: Vec<Box<dyn Handler + 's>>,

    // First line
    separator: Option<ProgressBar>,

    /// Keep track of the handler could while applying them. Usefull for
    /// debugging.
    pub handlers_len: usize,
}

impl<'s> State<'s> {
    pub fn new(cmd: &'s NixCommand) -> Self {
        let multi_progress = Rc::new(MultiProgress::default());

        let mut state = Self {
            cmd,
            multi_progress,
            handlers: Vec::new(),
            separator: None,
            handlers_len: 0,
        };

        if cmd.args.pix_debug {
            state.plug(DebugHandler::new(&state));
        }

        state.plug(handle_new_builds);
        state.plug(handle_new_download);
        state.plug(handle_new_message);
        state
    }
}

impl<'s> State<'s> {
    pub fn handle(&mut self, action: &Action) {
        // Move out handlers to allow borrowing self
        let mut prev_handlers = std::mem::take(&mut self.handlers);
        prev_handlers.retain_mut(|h| h.handle(self, action) == HandlerResult::Continue);

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
                .with_message("·".repeat(512));

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

    log_stream
        .try_for_each(|line| {
            match line {
                WrappedLine::StdOut(line) => state.println(line),
                WrappedLine::StdErr(line) => {
                    if let Some(action_str) = line.strip_prefix("@nix ") {
                        let action =
                            serde_json::from_str(action_str).expect("invalid JSON in action");
                        state.handle(&action);
                    } else {
                        state.println(line);
                    }
                }
            };

            async { Ok(()) }
        })
        .await
}
