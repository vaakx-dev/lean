use alacritty_terminal::index::Point;
use alacritty_terminal::term::TermMode;
use gpui_kit::Modifiers;

const WHEEL_UP: u8 = 64;
const WHEEL_DOWN: u8 = 65;
const NORMAL_OFFSET: u8 = 32;

pub(super) fn wheel_report_bytes(
    lines: i32,
    cell: Point<usize>,
    modifiers: &Modifiers,
    mode: TermMode,
) -> Vec<u8> {
    let direction = if lines > 0 { WHEEL_UP } else { WHEEL_DOWN };
    let button = direction + modifier_bits(modifiers);
    let report = if mode.contains(TermMode::SGR_MOUSE) {
        sgr_report(button, cell)
    } else {
        normal_report(button, cell, mode.contains(TermMode::UTF8_MOUSE))
    };
    let repeat = usize::try_from(lines.unsigned_abs()).unwrap_or(0);
    report.repeat(repeat)
}

fn modifier_bits(modifiers: &Modifiers) -> u8 {
    4 * u8::from(modifiers.shift) + 8 * u8::from(modifiers.alt) + 16 * u8::from(modifiers.control)
}

fn sgr_report(button: u8, cell: Point<usize>) -> Vec<u8> {
    format!("\x1b[<{button};{};{}M", cell.column.0 + 1, cell.line + 1).into_bytes()
}

fn normal_report(button: u8, cell: Point<usize>, utf8: bool) -> Vec<u8> {
    let mut report = b"\x1b[M".to_vec();
    report.push(button + NORMAL_OFFSET);
    for coordinate in [cell.column.0, cell.line] {
        let value = coordinate + 1 + usize::from(NORMAL_OFFSET);
        if !push_coordinate(&mut report, value, utf8) {
            return Vec::new();
        }
    }
    report
}

fn push_coordinate(report: &mut Vec<u8>, value: usize, utf8: bool) -> bool {
    if !utf8 {
        let Ok(byte) = u8::try_from(value) else {
            return false;
        };
        report.push(byte);
        return true;
    }
    let Some(character) = u32::try_from(value).ok().and_then(char::from_u32) else {
        return false;
    };
    let mut buffer = [0; 4];
    report.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
    true
}
