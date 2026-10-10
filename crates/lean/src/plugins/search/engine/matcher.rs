mod word_edges;

use std::ops::Range;

use gpui_kit::SharedString;
use regex::{Regex, RegexBuilder};

use crate::plugins::search::query::SearchOptions;
use word_edges::{is_whole_word, next_char_start};

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
            .build()
            .map_err(|error| error_message(&error))?;
        Ok(Self {
            regex,
            whole_word: options.whole_word,
        })
    }

    pub(super) fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    pub(super) fn has_match_in_line(&self, line: &str) -> bool {
        self.matches(line).next().is_some()
    }

    pub(super) fn find_in_line(&self, line: &str) -> Vec<Range<usize>> {
        self.matches(line).collect()
    }

    fn matches<'a>(&'a self, line: &'a str) -> impl Iterator<Item = Range<usize>> + 'a {
        let mut start = 0;
        std::iter::from_fn(move || {
            while start <= line.len() {
                let range = self.regex.find_at(line, start)?.range();
                if !range.is_empty() && (!self.whole_word || is_whole_word(line, &range)) {
                    start = range.end;
                    return Some(range);
                }
                start = next_char_start(line, range.start);
            }
            None
        })
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
