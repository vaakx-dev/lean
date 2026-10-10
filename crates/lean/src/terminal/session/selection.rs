use alacritty_terminal::index::{Point, Side};
use alacritty_terminal::selection::{Selection, SelectionType};

use super::TerminalSession;

impl TerminalSession {
    pub(crate) fn start_selection(&self, kind: SelectionType, point: Point, side: Side) {
        self.term.lock().selection = Some(Selection::new(kind, point, side));
    }

    pub(crate) fn extend_selection(&self, point: Point, side: Side) -> bool {
        let mut term = self.term.lock();
        match term.selection.as_mut() {
            Some(selection) => {
                selection.update(point, side);
                true
            }
            None => false,
        }
    }

    pub(crate) fn clear_selection(&self) {
        self.term.lock().selection = None;
    }

    pub(crate) fn selected_text(&self) -> Option<String> {
        self.term
            .lock()
            .selection_to_string()
            .filter(|text| !text.is_empty())
    }

    pub(crate) fn display_offset(&self) -> usize {
        self.term.lock().grid().display_offset()
    }
}
