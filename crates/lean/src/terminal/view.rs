mod bell;
mod blink;
mod events;
mod selection;
mod startup;
mod status;

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use alacritty_terminal::event::Event;
use alacritty_terminal::term::Config;
use futures::StreamExt as _;
use futures::channel::mpsc::{UnboundedReceiver, unbounded};
use gpui_kit::component::v_flex;
use gpui_kit::*;

use super::input::KEY_CONTEXT;
use super::launch::TerminalLaunch;
use super::render::{CursorAppearance, TerminalAppearance, terminal_canvas};
use super::session::{TerminalSession, shell_name, term_config};
use crate::settings::Settings;
use bell::Bell;
use blink::CursorBlink;
use startup::StartupCommand;
use status::TerminalStatus;

pub(crate) enum TerminalEvent {
    Exited,
    TitleChanged,
}

pub(crate) struct TerminalView {
    working_directory: PathBuf,
    launch: TerminalLaunch,
    startup: StartupCommand,
    shell_name: SharedString,
    title: Option<SharedString>,
    session: Option<Rc<TerminalSession>>,
    term_config: Config,
    status: TerminalStatus,
    pub(super) focus_handle: FocusHandle,
    pub(super) scroll_remainder: f32,
    grid_origin: Rc<Cell<Point<Pixels>>>,
    selecting: bool,
    blink: CursorBlink,
    bell: Bell,
    _event_task: Option<Task<()>>,
    _settings: Subscription,
}

impl EventEmitter<TerminalEvent> for TerminalView {}

impl TerminalView {
    pub(crate) fn new(
        working_directory: PathBuf,
        launch: TerminalLaunch,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut view = Self {
            working_directory,
            launch,
            startup: StartupCommand::default(),
            shell_name: shell_name(cx),
            title: None,
            session: None,
            term_config: term_config(cx),
            status: TerminalStatus::Running,
            focus_handle: cx.focus_handle(),
            scroll_remainder: 0.,
            grid_origin: Rc::default(),
            selecting: false,
            blink: CursorBlink::default(),
            bell: Bell::default(),
            _event_task: None,
            _settings: cx.observe_global::<Settings>(|view, cx| view.apply_settings(cx)),
        };
        view.start(cx);
        view
    }

    pub(super) fn start(&mut self, cx: &mut Context<Self>) {
        let (sender, receiver) = unbounded();
        self.shell_name = shell_name(cx);
        self.term_config = term_config(cx);
        self.set_title(None, cx);
        let spawned = TerminalSession::spawn(
            &self.working_directory,
            &self.launch.environment,
            sender,
            cx,
        );
        match spawned {
            Ok(session) => {
                self.session = Some(Rc::new(session));
                self.status = TerminalStatus::Running;
                self._event_task = Some(Self::listen(receiver, cx));
                self.queue_startup_command(cx);
            }
            Err(error) => {
                log::error!("failed to start the terminal shell: {error}");
                self.session = None;
                self.status = TerminalStatus::Failed(error.to_string().into());
                self._event_task = None;
            }
        }
        self.scroll_remainder = 0.;
        self.selecting = false;
        cx.notify();
    }

    pub(crate) fn shell_name(&self) -> SharedString {
        self.shell_name.clone()
    }

    pub(crate) fn title(&self) -> Option<SharedString> {
        self.title.clone()
    }

    pub(crate) fn is_busy(&self) -> bool {
        self.is_running()
            && self
                .session
                .as_ref()
                .is_some_and(|session| session.is_busy())
    }

    pub(super) fn session(&self) -> Option<Rc<TerminalSession>> {
        self.session.clone()
    }

    pub(super) fn is_running(&self) -> bool {
        matches!(self.status, TerminalStatus::Running)
    }

    pub(super) fn mark_exited(&mut self, code: Option<i32>, cx: &mut Context<Self>) {
        if !self.is_running() {
            return;
        }
        self.status = TerminalStatus::Exited(code);
        cx.emit(TerminalEvent::Exited);
        cx.notify();
    }

    pub(super) fn set_title(&mut self, title: Option<SharedString>, cx: &mut Context<Self>) {
        let title = title.filter(|title| !title.trim().is_empty());
        if self.title == title {
            return;
        }
        self.title = title;
        cx.emit(TerminalEvent::TitleChanged);
    }

    fn listen(mut receiver: UnboundedReceiver<Event>, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |view, cx| {
            while let Some(event) = receiver.next().await {
                let mut events = vec![event];
                while let Ok(event) = receiver.try_recv() {
                    events.push(event);
                }
                let handled = view.update(cx, |view, cx| {
                    for event in events {
                        view.handle_event(event, cx);
                    }
                });
                if handled.is_err() {
                    break;
                }
            }
        })
    }

    fn apply_settings(&mut self, cx: &mut Context<Self>) {
        let config = term_config(cx);
        if config != self.term_config {
            if let Some(session) = &self.session {
                session.set_config(config.clone());
            }
            self.term_config = config;
        }
        cx.notify();
    }

    fn cursor_appearance(&mut self, window: &Window, cx: &mut Context<Self>) -> CursorAppearance {
        let focused = self.focus_handle.is_focused(window);
        let cursor_shown = self.blink_phase_visible(focused, cx);
        if !self.is_running() {
            CursorAppearance::Hidden
        } else if !focused {
            CursorAppearance::Hollow
        } else if cursor_shown {
            CursorAppearance::Solid
        } else {
            CursorAppearance::Hidden
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
        let cursor = self.cursor_appearance(window, cx);
        let appearance = TerminalAppearance::new(cursor, cx);
        let bell_flash = self.render_bell(window, cx);
        let status = self.render_status(cx);
        v_flex()
            .id("terminal")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::key_down))
            .on_scroll_wheel(cx.listener(Self::scroll_wheel))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::selection_mouse_down))
            .on_mouse_move(cx.listener(Self::selection_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::selection_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::selection_mouse_up))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::copy_selection))
            .relative()
            .size_full()
            .overflow_hidden()
            .px_2()
            .py_1()
            .bg(appearance.background())
            .children(
                self.session
                    .clone()
                    .map(|session| terminal_canvas(session, appearance, self.grid_origin.clone())),
            )
            .children(status)
            .children(bell_flash)
    }
}
