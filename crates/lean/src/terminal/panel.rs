mod header;

use std::path::PathBuf;

use gpui_kit::component::v_flex;
use gpui_kit::*;

use super::view::{TerminalEvent, TerminalView};

pub(crate) enum TerminalPanelEvent {
    Hide,
}

struct TerminalTab {
    view: Entity<TerminalView>,
    _subscription: Subscription,
}

pub(crate) struct TerminalPanel {
    working_directory: PathBuf,
    tabs: Vec<TerminalTab>,
    active: usize,
}

impl EventEmitter<TerminalPanelEvent> for TerminalPanel {}

impl TerminalPanel {
    pub(crate) fn new(working_directory: PathBuf, cx: &mut Context<Self>) -> Self {
        let mut panel = Self {
            working_directory,
            tabs: Vec::new(),
            active: 0,
        };
        panel.add_terminal(cx);
        panel
    }

    pub(crate) fn focus_active(&self, window: &mut Window, cx: &mut App) {
        if let Some(tab) = self.tabs.get(self.active) {
            let focus_handle = tab.view.focus_handle(cx);
            window.focus(&focus_handle, cx);
        }
    }

    pub(crate) fn contains_focus(&self, window: &Window, cx: &App) -> bool {
        self.tabs
            .iter()
            .any(|tab| tab.view.focus_handle(cx).contains_focused(window, cx))
    }

    fn add_terminal(&mut self, cx: &mut Context<Self>) {
        let working_directory = self.working_directory.clone();
        let view = cx.new(|cx| TerminalView::new(working_directory, cx));
        let subscription = cx.subscribe(&view, |_, _, event: &TerminalEvent, cx| match event {
            TerminalEvent::Exited => cx.notify(),
        });
        self.tabs.push(TerminalTab {
            view,
            _subscription: subscription,
        });
        self.active = self.tabs.len() - 1;
        cx.notify();
    }

    fn new_terminal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.add_terminal(cx);
        self.focus_active(window, cx);
    }

    fn activate(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        self.active = index;
        self.focus_active(window, cx);
        cx.notify();
    }

    fn close(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.tabs.len() <= 1 || index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);
        if self.active > index || self.active >= self.tabs.len() {
            self.active = self.active.saturating_sub(1);
        }
        self.focus_active(window, cx);
        cx.notify();
    }
}

impl Render for TerminalPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().size_full().child(self.render_header(cx)).children(
            self.tabs
                .get(self.active)
                .map(|tab| div().flex_1().min_h_0().child(tab.view.clone())),
        )
    }
}
