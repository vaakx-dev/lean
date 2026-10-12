use alacritty_terminal::index::{Column, Point as GridPoint, Side};
use alacritty_terminal::selection::SelectionType;
use alacritty_terminal::term::viewport_to_point;
use gpui_kit::*;

use super::TerminalView;
use crate::terminal::input::CopySelection;
use crate::terminal::session::{GridSize, TerminalSession};
use crate::terminal::settings::COPY_ON_SELECT;

impl TerminalView {
    pub(super) fn selection_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);
        let Some(session) = self.session() else {
            return;
        };
        let (point, side) = self.grid_point(event.position, &session);
        let extended = event.modifiers.shift && session.extend_selection(point, side);
        if !extended {
            session.start_selection(selection_type(event.click_count), point, side);
        }
        self.selecting = true;
        cx.notify();
    }

    pub(super) fn selection_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.selecting {
            return;
        }
        if event.pressed_button != Some(MouseButton::Left) {
            self.finish_selection(cx);
            return;
        }
        let Some(session) = self.session() else {
            return;
        };
        let (point, side) = self.grid_point(event.position, &session);
        if session.extend_selection(point, side) {
            cx.notify();
        }
    }

    pub(super) fn selection_mouse_up(
        &mut self,
        _: &MouseUpEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.finish_selection(cx);
    }

    pub(super) fn copy_selection(
        &mut self,
        _: &CopySelection,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.write_selection_to_clipboard(cx);
    }

    fn finish_selection(&mut self, cx: &mut Context<Self>) {
        if !std::mem::replace(&mut self.selecting, false) {
            return;
        }
        if COPY_ON_SELECT.get(cx) {
            self.write_selection_to_clipboard(cx);
        }
    }

    fn write_selection_to_clipboard(&self, cx: &mut App) {
        if let Some(text) = self.session().and_then(|session| session.selected_text()) {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    pub(in crate::terminal) fn viewport_point(
        &self,
        position: Point<Pixels>,
        size: &GridSize,
    ) -> (GridPoint<usize>, Side) {
        let offset = position - self.grid_origin.get();
        let (column, side) = column_at(offset.x, size);
        let line = cell_index(offset.y / size.line_height, size.lines);
        (GridPoint::new(line, Column(column)), side)
    }

    fn grid_point(&self, position: Point<Pixels>, session: &TerminalSession) -> (GridPoint, Side) {
        let (viewport, side) = self.viewport_point(position, &session.size());
        (viewport_to_point(session.display_offset(), viewport), side)
    }
}

fn selection_type(click_count: usize) -> SelectionType {
    match click_count {
        0 | 1 => SelectionType::Simple,
        2 => SelectionType::Semantic,
        _ => SelectionType::Lines,
    }
}

fn column_at(x: Pixels, size: &GridSize) -> (usize, Side) {
    let cells = x / size.cell_width;
    if cells >= size.columns as f32 {
        return (size.columns.saturating_sub(1), Side::Right);
    }
    let side = if cells.fract() > 0.5 {
        Side::Right
    } else {
        Side::Left
    };
    (cell_index(cells, size.columns), side)
}

fn cell_index(cells: f32, count: usize) -> usize {
    (cells.max(0.).floor() as usize).min(count.saturating_sub(1))
}
