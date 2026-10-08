use std::{cell::RefCell, ops::Range, sync::Arc};

use gpui::HighlightStyle;
use ropey::{Rope, extra::esoterica::ropes_are_instances};

use super::SharedHighlightStyleResolver;

pub(super) type SyntaxStyles = Vec<(Range<usize>, HighlightStyle)>;

pub(super) struct SyntaxStyleKey {
    pub(super) text: Rope,
    pub(super) highlighter: usize,
    pub(super) generation: u64,
    pub(super) resolver: SharedHighlightStyleResolver,
    pub(super) line_groups: Vec<(Range<usize>, bool)>,
}

impl SyntaxStyleKey {
    fn matches(&self, other: &Self) -> bool {
        self.generation != 0
            && self.generation == other.generation
            && self.highlighter == other.highlighter
            && Arc::ptr_eq(&self.resolver, &other.resolver)
            && self.line_groups == other.line_groups
            && ropes_are_instances(&self.text, &other.text)
    }
}

#[derive(Default)]
pub(super) struct SyntaxStyleCache {
    last: RefCell<Option<(SyntaxStyleKey, SyntaxStyles)>>,
}

impl SyntaxStyleCache {
    pub(super) fn styles(
        &self,
        key: SyntaxStyleKey,
        compute: impl FnOnce(&[(Range<usize>, bool)]) -> SyntaxStyles,
    ) -> SyntaxStyles {
        if let Some((_, styles)) = self
            .last
            .borrow()
            .as_ref()
            .filter(|(last_key, _)| last_key.matches(&key))
        {
            return styles.clone();
        }
        let styles = compute(&key.line_groups);
        *self.last.borrow_mut() = (key.generation != 0).then(|| (key, styles.clone()));
        styles
    }
}
