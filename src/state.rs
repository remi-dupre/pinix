use std::rc::Rc;

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::action::Action;

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
    pub multi_progress: Rc<MultiProgress>,
    pub handlers: Vec<Box<dyn Handler + 's>>,
    pub separator: ProgressBar,

    /// Keep track of the handler could while applying them. Usefull for
    /// debugging.
    pub handlers_len: usize,
}

impl Default for State<'_> {
    fn default() -> Self {
        let multi_progress = Rc::new(MultiProgress::default());

        let separator = ProgressBar::new_spinner()
            .with_style(
                ProgressStyle::default_spinner()
                    .template("{wide_msg:^}")
                    .expect("invalid template"),
            )
            .with_message(style("-".repeat(512)).dim().to_string());

        let separator = multi_progress.add(separator);
        separator.set_length(0);

        Self {
            multi_progress,
            handlers: Vec::new(),
            separator,
            handlers_len: 0,
        }
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

    pub fn println(&self, msg: impl AsRef<str>) {
        self.multi_progress
            .println(msg)
            .expect("could not print line")
    }
}
