use std::ops::Range;

use gpui_kit::SharedString;
use regex::{Regex, RegexBuilder};

use super::lines::line_break;
use crate::plugins::search::query::SearchOptions;

pub(super) struct Matcher {
    regex: Regex,
    whole_word: bool,
}

impl Matcher {
    pub(super) fn new(
        pattern: &str,
        options: SearchOptions,
        smart_case: bool,
    ) -> Result<Self, SharedString> {
        let source = if options.regex {
            pattern.to_string()
        } else {
            regex::escape(pattern)
        };
        let case_sensitive =
            options.match_case || (smart_case && pattern.chars().any(char::is_uppercase));
        let regex = RegexBuilder::new(&source)
            .case_insensitive(!case_sensitive)
            .multi_line(true)
            .crlf(true)
            .build()
            .map_err(|error| error_message(&error))?;
        Ok(Self {
            regex,
            whole_word: options.whole_word,
        })
    }

    pub(super) fn find_iter<'a>(
        &'a self,
        text: &'a str,
    ) -> impl Iterator<Item = Range<usize>> + 'a {
        let mut start = 0;
        let mut line_end = 0..0;
        std::iter::from_fn(move || {
            while start <= text.len() {
                let found = self.regex.find_at(text, start)?.range();
                if found.start >= line_end.end {
                    line_end = line_break(text, found.start);
                }
                let Some(range) = self.within_line(text, found, line_end.start) else {
                    start = line_end.end;
                    continue;
                };
                if !range.is_empty() && (!self.whole_word || is_whole_word(text, &range)) {
                    start = range.end;
                    return Some(range);
                }
                start = next_char_start(text, range.start);
            }
            None
        })
    }

    fn within_line(
        &self,
        text: &str,
        found: Range<usize>,
        line_end: usize,
    ) -> Option<Range<usize>> {
        if found.end <= line_end {
            return Some(found);
        }
        if found.start >= line_end {
            return None;
        }
        self.regex
            .find_at(&text[..line_end], found.start)
            .map(|found| found.range())
    }
}

fn error_message(error: &regex::Error) -> SharedString {
    let message = error.to_string();
    let detail = message
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("invalid pattern")
        .trim_start_matches("error: ");
    format!("Invalid regular expression: {detail}").into()
}

fn is_whole_word(text: &str, range: &Range<usize>) -> bool {
    let found = &text[range.clone()];
    let before = text[..range.start].chars().next_back();
    let after = text[range.end..].chars().next();
    let starts_cleanly =
        !(found.chars().next().is_some_and(is_word_char) && before.is_some_and(is_word_char));
    let ends_cleanly =
        !(found.chars().next_back().is_some_and(is_word_char) && after.is_some_and(is_word_char));
    starts_cleanly && ends_cleanly
}

fn next_char_start(text: &str, index: usize) -> usize {
    index + text[index..].chars().next().map_or(1, char::len_utf8)
}

fn is_word_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}
