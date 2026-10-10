use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::input::{EditorState, SharedHighlightStyleResolver};
use gpui_kit::*;

use super::Minimap;
use super::chunks::{BuiltChunks, ChunkJob, Chunks};

const EDIT_DEBOUNCE: Duration = Duration::from_millis(120);

pub(super) struct Styling {
    language: SharedString,
    generation: u64,
    resolver: SharedHighlightStyleResolver,
}

impl Styling {
    pub(super) fn read(editor: &EditorState) -> Self {
        Self {
            language: editor.language_name(),
            generation: editor.highlight_generation(),
            resolver: editor.highlight_style_resolver(),
        }
    }
}

impl PartialEq for Styling {
    fn eq(&self, other: &Self) -> bool {
        self.language == other.language
            && self.generation == other.generation
            && Arc::ptr_eq(&self.resolver, &other.resolver)
    }
}

#[derive(Clone, Copy)]
enum Phase {
    Idle,
    Waiting(Duration),
    Building,
}

pub(super) struct Refresh {
    version: u64,
    edited: bool,
    styling: Styling,
    phase: Phase,
    task: Task<()>,
}

impl Refresh {
    pub(super) fn new(styling: Styling) -> Self {
        Self {
            version: 0,
            edited: false,
            styling,
            phase: Phase::Idle,
            task: Task::ready(()),
        }
    }

    pub(super) fn text_changed(&mut self) {
        self.version += 1;
        self.edited = true;
    }

    pub(super) fn restyle(&mut self) {
        self.version += 1;
    }

    pub(super) fn sync_styling(&mut self, styling: Styling) -> bool {
        if styling == self.styling {
            return false;
        }
        self.styling = styling;
        self.restyle();
        true
    }

    fn delay_for(
        &self,
        chunks: &Chunks,
        visible: &Range<usize>,
        wanted: &Range<usize>,
    ) -> Option<Duration> {
        let delay = self.wanted_delay(chunks, visible, wanted)?;
        match self.phase {
            Phase::Idle => Some(delay),
            Phase::Waiting(pending) => (delay < pending).then_some(delay),
            Phase::Building => None,
        }
    }

    fn wanted_delay(
        &self,
        chunks: &Chunks,
        visible: &Range<usize>,
        wanted: &Range<usize>,
    ) -> Option<Duration> {
        if chunks.has_missing(visible) {
            return Some(Duration::ZERO);
        }
        if !chunks.needs_build(wanted, self.version) {
            return None;
        }
        Some(if self.edited {
            EDIT_DEBOUNCE
        } else {
            Duration::ZERO
        })
    }
}

impl Minimap {
    pub(super) fn refresh_if_needed(&mut self, cx: &mut Context<Self>) {
        let (Some(visible), Some(wanted)) = (self.visible_lines(), self.wanted_lines()) else {
            return;
        };
        if let Some(delay) = self.refresh.delay_for(&self.chunks, &visible, &wanted) {
            self.schedule_refresh(delay, cx);
        }
    }

    fn schedule_refresh(&mut self, delay: Duration, cx: &mut Context<Self>) {
        self.refresh.phase = Phase::Waiting(delay);
        self.refresh.task = cx.spawn(async move |minimap, cx| {
            cx.background_executor().timer(delay).await;
            let Ok(Some(job)) = minimap.update(cx, |minimap, cx| minimap.start_chunk_job(cx))
            else {
                return;
            };
            let built = cx.background_spawn(async move { job.run() }).await;
            let update = minimap.update(cx, |minimap, cx| minimap.show_chunks(built, cx));
            if let Err(error) = update {
                log::debug!("minimap released before its refresh finished: {error:#}");
            }
        });
    }

    fn start_chunk_job(&mut self, cx: &mut Context<Self>) -> Option<ChunkJob> {
        self.refresh.phase = Phase::Idle;
        let wanted = self.wanted_lines()?;
        let version = self.refresh.version;
        let outdated = self.chunks.outdated(&wanted, version);
        if outdated.is_empty() {
            return None;
        }
        self.refresh.phase = Phase::Building;
        self.refresh.edited = false;
        let foreground = cx.theme().foreground;
        Some(ChunkJob::new(
            self.editor.read(cx),
            outdated,
            version,
            foreground,
            &self.style,
        ))
    }

    fn show_chunks(&mut self, built: BuiltChunks, cx: &mut Context<Self>) {
        self.refresh.phase = Phase::Idle;
        let visible = self.visible_lines().unwrap_or_default();
        if self.chunks.insert(built, &visible) {
            cx.notify();
        }
        self.request_refresh(cx);
    }
}
