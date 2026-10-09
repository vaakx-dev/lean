#[derive(Clone, Copy, Default)]
pub(super) struct SearchOptions {
    pub(super) match_case: bool,
    pub(super) whole_word: bool,
    pub(super) regex: bool,
}

pub(super) struct SearchQuery {
    pub(super) pattern: String,
    pub(super) options: SearchOptions,
    pub(super) include: String,
    pub(super) exclude: String,
}
