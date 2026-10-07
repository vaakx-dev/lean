mod grid_size;
mod listener;
mod shell;

use std::borrow::Cow;
use std::cell::Cell;
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

pub(crate) use grid_size::GridSize;
use listener::EventProxy;
pub(crate) use shell::shell_name;

const WINDOW_ID: u64 = 0;

pub(crate) struct TerminalSession {
    term: Arc<FairMutex<Term<EventProxy>>>,
    sender: EventLoopSender,
    size: Cell<GridSize>,
}

impl TerminalSession {
    pub(crate) fn spawn(
        working_directory: &Path,
        events: UnboundedSender<Event>,
    ) -> io::Result<Self> {
        let size = GridSize::default();
        let listener = EventProxy::new(events);
        let options = shell::shell_options(working_directory);
        let pty = tty::new(&options, size.window_size(), WINDOW_ID)?;
        let term = Term::new(Config::default(), &size, listener.clone());
        let term = Arc::new(FairMutex::new(term));
        let event_loop = EventLoop::new(term.clone(), listener, pty, options.drain_on_exit, false)?;
        let sender = event_loop.channel();
        event_loop.spawn();
        Ok(Self {
            term,
            sender,
            size: Cell::new(size),
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
