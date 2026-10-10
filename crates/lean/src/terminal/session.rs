mod config;
mod grid_size;
mod listener;
mod process;
mod selection;
mod shell;

use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::io;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::Arc;

use alacritty_terminal::event::Event;
use alacritty_terminal::event_loop::{EventLoop, EventLoopSender, Msg};
use alacritty_terminal::grid::Scroll;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::tty;
use futures::channel::mpsc::UnboundedSender;
use gpui_kit::App;

pub(crate) use config::term_config;
pub(crate) use grid_size::GridSize;
use listener::EventProxy;
use process::ShellProcess;
pub(crate) use shell::{ShellKind, default_shell_program, shell_kind, shell_name};

const WINDOW_ID: u64 = 0;

pub(crate) struct TerminalSession {
    term: Arc<FairMutex<Term<EventProxy>>>,
    sender: EventLoopSender,
    size: Cell<GridSize>,
    process: ShellProcess,
}

impl TerminalSession {
    pub(crate) fn spawn(
        working_directory: &Path,
        environment: &HashMap<String, String>,
        events: UnboundedSender<Event>,
        cx: &App,
    ) -> io::Result<Self> {
        let size = GridSize::default();
        let listener = EventProxy::new(events);
        let options = shell::shell_options(working_directory, environment, cx);
        let pty = tty::new(&options, size.window_size(), WINDOW_ID)?;
        let process = ShellProcess::of(&pty);
        let term = Term::new(term_config(cx), &size, listener.clone());
        let term = Arc::new(FairMutex::new(term));
        let event_loop = EventLoop::new(term.clone(), listener, pty, options.drain_on_exit, false)?;
        let sender = event_loop.channel();
        event_loop.spawn();
        Ok(Self {
            term,
            sender,
            size: Cell::new(size),
            process,
        })
    }

    pub(crate) fn lock(&self) -> impl DerefMut<Target = Term<EventProxy>> + '_ {
        self.term.lock()
    }

    pub(crate) fn mode(&self) -> TermMode {
        *self.term.lock().mode()
    }

    pub(crate) fn size(&self) -> GridSize {
        self.size.get()
    }

    pub(crate) fn cursor_blinks(&self) -> bool {
        self.term.lock().cursor_style().blinking
    }

    pub(crate) fn is_busy(&self) -> bool {
        self.process.has_children()
    }

    pub(crate) fn set_config(&self, config: Config) {
        self.term.lock().set_options(config);
    }

    pub(crate) fn write(&self, bytes: impl Into<Cow<'static, [u8]>>) {
        let bytes = bytes.into();
        if !bytes.is_empty() {
            self.send(Msg::Input(bytes));
        }
    }

    pub(crate) fn scroll(&self, scroll: Scroll) {
        self.term.lock().scroll_display(scroll);
    }

    pub(crate) fn resize(&self, size: GridSize) {
        if self.size.replace(size) == size {
            return;
        }
        self.term.lock().resize(size);
        self.send(Msg::Resize(size.window_size()));
    }

    fn send(&self, message: Msg) {
        if let Err(error) = self.sender.send(message) {
            log::debug!("terminal event loop is not running: {error}");
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.send(Msg::Shutdown);
    }
}
