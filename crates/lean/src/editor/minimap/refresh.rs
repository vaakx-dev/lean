use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::highlighter::{HighlightTheme, SyntaxHighlighter};
use gpui_kit::component::input::Rope;
use gpui_kit::*;

use super::Minimap;
use super::document::Document;

pub(super) const EDIT_REFRESH_DELAY: Duration = Duration::from_millis(300);

#[derive(Clone)]
pub(super) struct Syntax {
    language: SharedString,
    highlighter: Arc<Mutex<Option<SyntaxHighlighter>>>,
}

impl Syntax {
    pub(super) fn new(language: SharedString) -> Self {
        Self {
            language,
            highlighter: Arc::default(),
        }
    }

    fn styles(&self, text: &Rope, theme: &HighlightTheme) -> Vec<(Range<usize>, HighlightStyle)> {
        let mut slot = self
            .highlighter
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let highlighter = slot.get_or_insert_with(|| SyntaxHighlighter::new(&self.language));
        highlighter.update(None, text, None);
        highlighter.styles(&(0..text.len()), theme)
    }
}

impl Minimap {
    pub(super) fn schedule_refresh(&mut self, delay: Duration, cx: &mut Context<Self>) {
        let text = self.editor.read(cx).text().clone();
        let theme = cx.theme();
        let foreground = theme.foreground;
        let highlight_theme = theme.highlight_theme.clone();
        let syntax = self.syntax.clone();
        self._refresh_task = cx.spawn(async move |minimap, cx| {
            cx.background_executor().timer(delay).await;
            let document = cx
                .background_spawn(async move {
                    let styles = syntax.styles(&text, &highlight_theme);
                    Document::build(&text, &styles, foreground)
                })
                .await;
            let update = minimap.update(cx, |minimap, cx| {
                minimap.document = Rc::new(document);
                cx.notify();
            });
            if let Err(error) = update {
                log::debug!("minimap released before its refresh finished: {error:#}");
            }
        });
    }
}
