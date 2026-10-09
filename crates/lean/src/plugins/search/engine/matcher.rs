use std::ops::Range;

use gpui_kit::SharedString;
use regex::{Regex, RegexBuilder};

use crate::plugins::search::query::SearchOptions;

pub(super) struct Matcher {
    regex: Regex,
}

impl Matcher {
    pub(super) fn new(pattern: &str, options: SearchOptions) -> Result<Self, SharedString> {
        let source = if options.regex {
            pattern.to_string()
        } else {
            regex::escape(pattern)
        };
        let source = if options.whole_word {
            format!(r"\b(?:{source})\b")
        } else {
            source
        };
        let regex = RegexBuilder::new(&source)
            .case_insensitive(!options.match_case)
            .build()
            .map_err(|error| error_message(&error))?;
        Ok(Self { regex })
    }

    pub(super) fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    pub(super) fn find_in_line(&self, line: &str) -> Vec<Range<usize>> {
        self.regex
            .find_iter(line)
            .map(|found| found.range())
            .filter(|range| !range.is_empty())
            .collect()
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
