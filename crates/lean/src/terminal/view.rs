use std::path::PathBuf;
use std::rc::Rc;

use alacritty_terminal::event::Event;
use futures::StreamExt as _;
use futures::channel::mpsc::{UnboundedReceiver, unbounded};
use gpui_kit::component::{ActiveTheme as _, v_flex};
use gpui_kit::*;

use super::input::KEY_CONTEXT;
use super::render::{CursorAppearance, TerminalAppearance, terminal_canvas};
use super::session::{TerminalSession, shell_name};

const EXITED_MESSAGE: &str = "Process exited. Press Enter to restart.";

pub(crate) enum TerminalEvent {
    Exited,
}

enum TerminalStatus {
    Running,
    Exited,
    Failed(SharedString),
}

pub(crate) struct TerminalView {
    working_directory: PathBuf,
    shell_name: SharedString,
    session: Option<Rc<TerminalSession>>,
    status: TerminalStatus,
    pub(super) focus_handle: FocusHandle,
    pub(super) scroll_remainder: f32,
    _event_task: Option<Task<()>>,
}

impl EventEmitter<TerminalEvent> for TerminalView {}

impl TerminalView {
    pub(crate) fn new(working_directory: PathBuf, cx: &mut Context<Self>) -> Self {
        let mut view = Self {
            working_directory,
            shell_name: shell_name(),
            session: None,
            status: TerminalStatus::Running,
            focus_handle: cx.focus_handle(),
            scroll_remainder: 0.,
            _event_task: None,
        };
        view.start(cx);
        view
    }

    pub(super) fn start(&mut self, cx: &mut Context<Self>) {
        let (sender, receiver) = unbounded();
        match TerminalSession::spawn(&self.working_directory, sender) {
            Ok(session) => {
                self.session = Some(Rc::new(session));
                self.status = TerminalStatus::Running;
                self._event_task = Some(Self::listen(receiver, cx));
            }
            Err(error) => {
                log::error!("failed to start the terminal shell: {error}");
                self.session = None;
                self.status =
                    TerminalStatus::Failed(format!("Failed to start the shell: {error}").into());
                self._event_task = None;
            }
        }
        self.scroll_remainder = 0.;
        cx.notify();
    }

    pub(crate) fn shell_name(&self) -> SharedString {
        self.shell_name.clone()
    }

    pub(super) fn session(&self) -> Option<Rc<TerminalSession>> {
        self.session.clone()
    }

    pub(super) fn is_running(&self) -> bool {
        matches!(self.status, TerminalStatus::Running)
    }

    pub(super) fn mark_exited(&mut self, cx: &mut Context<Self>) {
        if !self.is_running() {
            return;
        }
        self.status = TerminalStatus::Exited;
        cx.emit(TerminalEvent::Exited);
        cx.notify();
    }

    fn listen(mut receiver: UnboundedReceiver<Event>, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |view, cx| {
            while let Some(event) = receiver.next().await {
                if let Err(error) = view.update(cx, |view, cx| view.handle_event(event, cx)) {
                    log::debug!("terminal view dropped while its shell was running: {error:#}");
                    break;
                }
            }
        })
    }

    fn cursor_appearance(&self, window: &Window) -> CursorAppearance {
        if !self.is_running() {
            CursorAppearance::Hidden
        } else if self.focus_handle.is_focused(window) {
            CursorAppearance::Solid
        } else {
            CursorAppearance::Hollow
        }
    }

    fn status_message(&self) -> Option<SharedString> {
        match &self.status {
            TerminalStatus::Running => None,
            TerminalStatus::Exited => Some(EXITED_MESSAGE.into()),
            TerminalStatus::Failed(message) => Some(message.clone()),
        }
    }
}

impl Focusable for TerminalView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let appearance = TerminalAppearance::new(cx.theme(), self.cursor_appearance(window));
        let muted_foreground = cx.theme().muted_foreground;
        v_flex()
            .id("terminal")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::key_down))
            .on_scroll_wheel(cx.listener(Self::scroll_wheel))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::focus_on_click))
            .on_action(cx.listener(Self::paste))
            .size_full()
            .overflow_hidden()
            .px_2()
            .py_1()
            .bg(appearance.background())
            .children(
                self.session
                    .clone()
                    .map(|session| terminal_canvas(session, appearance)),
            )
            .children(self.status_message().map(|message| {
                div()
                    .flex_shrink_0()
                    .pt_1()
                    .text_xs()
                    .text_color(muted_foreground)
                    .child(message)
            }))
    }
}
