use std::ops::Range;

pub(super) fn is_whole_word(line: &str, range: &Range<usize>) -> bool {
    let found = &line[range.clone()];
    let before = line[..range.start].chars().next_back();
    let after = line[range.end..].chars().next();
    let starts_cleanly =
        !(found.chars().next().is_some_and(is_word_char) && before.is_some_and(is_word_char));
    let ends_cleanly =
        !(found.chars().next_back().is_some_and(is_word_char) && after.is_some_and(is_word_char));
    starts_cleanly && ends_cleanly
}

pub(super) fn next_char_start(line: &str, index: usize) -> usize {
    index + line[index..].chars().next().map_or(1, char::len_utf8)
}

fn is_word_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}
