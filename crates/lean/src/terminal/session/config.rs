use alacritty_terminal::term::{Config, Osc52};
use alacritty_terminal::vte::ansi::{CursorShape, CursorStyle};
use gpui_kit::App;

use crate::terminal::settings::{
    CURSOR_BLINK, CURSOR_SHAPE, ClipboardAccess, CursorShapeChoice, OSC52, SCROLLBACK,
};

pub(crate) fn term_config(cx: &App) -> Config {
    Config {
        scrolling_history: SCROLLBACK.get(cx) as usize,
        default_cursor_style: CursorStyle {
            shape: cursor_shape(CURSOR_SHAPE.get(cx)),
            blinking: CURSOR_BLINK.get(cx),
        },
        osc52: osc52(OSC52.get(cx)),
        ..Config::default()
    }
}

fn cursor_shape(choice: CursorShapeChoice) -> CursorShape {
    match choice {
        CursorShapeChoice::Block => CursorShape::Block,
        CursorShapeChoice::Beam => CursorShape::Beam,
        CursorShapeChoice::Underline => CursorShape::Underline,
    }
}

fn osc52(access: ClipboardAccess) -> Osc52 {
    match access {
        ClipboardAccess::Off => Osc52::Disabled,
        ClipboardAccess::Copy => Osc52::OnlyCopy,
        ClipboardAccess::CopyPaste => Osc52::CopyPaste,
    }
}
