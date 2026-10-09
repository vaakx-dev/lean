const ESCAPE: char = '\u{1b}';

pub(super) fn strip_ansi(text: &str) -> String {
    let mut plain = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(char) = chars.next() {
        if char != ESCAPE {
            plain.push(char);
            continue;
        }
        if chars.next() == Some('[') {
            for code in chars.by_ref() {
                if is_final_byte(code) {
                    break;
                }
            }
        }
    }
    plain
}

pub(super) fn incomplete_escape_start(text: &str) -> Option<usize> {
    let start = text.rfind(ESCAPE)?;
    let rest = &text[start + ESCAPE.len_utf8()..];
    let complete = match rest.strip_prefix('[') {
        Some(sequence) => sequence.chars().any(is_final_byte),
        None => !rest.is_empty(),
    };
    (!complete).then_some(start)
}

fn is_final_byte(code: char) -> bool {
    ('@'..='~').contains(&code)
}
