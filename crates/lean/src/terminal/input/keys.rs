use alacritty_terminal::grid::Scroll;
use alacritty_terminal::term::TermMode;
use gpui_kit::{Keystroke, Modifiers};

const ESCAPE: u8 = 0x1b;

pub(crate) fn keystroke_bytes(keystroke: &Keystroke, mode: TermMode) -> Option<Vec<u8>> {
    let modifiers = &keystroke.modifiers;
    if modifiers.platform {
        return None;
    }
    if let Some(bytes) = named_key_bytes(&keystroke.key, modifiers, mode) {
        return Some(bytes);
    }
    if modifiers.control {
        return control_bytes(&keystroke.key, modifiers.alt);
    }
    let text = keystroke
        .key_char
        .as_deref()
        .or_else(|| (keystroke.key.chars().count() == 1).then_some(keystroke.key.as_str()))?;
    Some(with_alt(text.as_bytes(), modifiers.alt))
}

pub(crate) fn history_scroll(keystroke: &Keystroke) -> Option<Scroll> {
    let modifiers = &keystroke.modifiers;
    if !modifiers.shift || modifiers.control || modifiers.alt || modifiers.platform {
        return None;
    }
    match keystroke.key.as_str() {
        "pageup" => Some(Scroll::PageUp),
        "pagedown" => Some(Scroll::PageDown),
        "home" => Some(Scroll::Top),
        "end" => Some(Scroll::Bottom),
        _ => None,
    }
}

pub(crate) fn paste_bytes(text: &str, mode: TermMode) -> Vec<u8> {
    if mode.contains(TermMode::BRACKETED_PASTE) {
        let sanitized = text.replace('\x1b', "");
        format!("\x1b[200~{sanitized}\x1b[201~").into_bytes()
    } else {
        text.replace("\r\n", "\r").replace('\n', "\r").into_bytes()
    }
}

pub(crate) fn alternate_scroll_bytes(lines: i32, mode: TermMode) -> Vec<u8> {
    let direction = if lines > 0 { 'A' } else { 'B' };
    let application_cursor = mode.contains(TermMode::APP_CURSOR);
    let repeat = usize::try_from(lines.unsigned_abs()).unwrap_or(0);
    cursor_key(direction, 1, application_cursor).repeat(repeat)
}

fn named_key_bytes(key: &str, modifiers: &Modifiers, mode: TermMode) -> Option<Vec<u8>> {
    let code = modifier_code(modifiers);
    let application_cursor = mode.contains(TermMode::APP_CURSOR);
    let alt = modifiers.alt;
    let bytes = match key {
        "enter" => with_alt(b"\r", alt),
        "backspace" if modifiers.control => with_alt(b"\x08", alt),
        "backspace" => with_alt(b"\x7f", alt),
        "tab" if modifiers.shift => b"\x1b[Z".to_vec(),
        "tab" => with_alt(b"\t", alt),
        "escape" => with_alt(b"\x1b", alt),
        "space" if modifiers.control => with_alt(b"\0", alt),
        "space" => with_alt(b" ", alt),
        "up" => cursor_key('A', code, application_cursor),
        "down" => cursor_key('B', code, application_cursor),
        "right" => cursor_key('C', code, application_cursor),
        "left" => cursor_key('D', code, application_cursor),
        "home" => cursor_key('H', code, application_cursor),
        "end" => cursor_key('F', code, application_cursor),
        "insert" => tilde_key(2, code),
        "delete" => tilde_key(3, code),
        "pageup" => tilde_key(5, code),
        "pagedown" => tilde_key(6, code),
        "f1" => function_key('P', code),
        "f2" => function_key('Q', code),
        "f3" => function_key('R', code),
        "f4" => function_key('S', code),
        "f5" => tilde_key(15, code),
        "f6" => tilde_key(17, code),
        "f7" => tilde_key(18, code),
        "f8" => tilde_key(19, code),
        "f9" => tilde_key(20, code),
        "f10" => tilde_key(21, code),
        "f11" => tilde_key(23, code),
        "f12" => tilde_key(24, code),
        _ => return None,
    };
    Some(bytes)
}

fn control_bytes(key: &str, alt: bool) -> Option<Vec<u8>> {
    let mut characters = key.chars();
    let character = characters.next()?;
    if characters.next().is_some() {
        return None;
    }
    let code = match character.to_ascii_lowercase() {
        letter @ 'a'..='z' => letter as u8 - b'a' + 1,
        '@' | '2' => 0,
        '[' | '3' => 0x1b,
        '\\' | '4' => 0x1c,
        ']' | '5' => 0x1d,
        '^' | '6' => 0x1e,
        '_' | '-' | '7' => 0x1f,
        '?' | '8' => 0x7f,
        _ => return None,
    };
    Some(with_alt(&[code], alt))
}

fn modifier_code(modifiers: &Modifiers) -> u8 {
    1 + u8::from(modifiers.shift) + 2 * u8::from(modifiers.alt) + 4 * u8::from(modifiers.control)
}

fn cursor_key(letter: char, code: u8, application_cursor: bool) -> Vec<u8> {
    let sequence = if code > 1 {
        format!("\x1b[1;{code}{letter}")
    } else if application_cursor {
        format!("\x1bO{letter}")
    } else {
        format!("\x1b[{letter}")
    };
    sequence.into_bytes()
}

fn tilde_key(number: u8, code: u8) -> Vec<u8> {
    let sequence = if code > 1 {
        format!("\x1b[{number};{code}~")
    } else {
        format!("\x1b[{number}~")
    };
    sequence.into_bytes()
}

fn function_key(letter: char, code: u8) -> Vec<u8> {
    let sequence = if code > 1 {
        format!("\x1b[1;{code}{letter}")
    } else {
        format!("\x1bO{letter}")
    };
    sequence.into_bytes()
}

fn with_alt(bytes: &[u8], alt: bool) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len() + 1);
    if alt {
        output.push(ESCAPE);
    }
    output.extend_from_slice(bytes);
    output
}
