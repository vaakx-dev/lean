use std::{
    cell::{Cell, RefCell},
    ops::{ControlFlow, Range},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use gpui::{HighlightStyle, SharedString, Task};
use gpui_base::input::{
    EditorState, FoldRange, HighlightStyleResolver, InputEdit as BaseInputEdit,
    InputHighlightSnapshot, InputHighlighter, InputHighlighterFactory, RopeExt as _,
};
use ropey::Rope;
use tree_sitter::{InputEdit, ParseOptions, Point, Tree};

use super::{
    LanguageRegistry, SyntaxHighlighter,
    highlighter::{SyntaxSnapshot, parse_input_bytes},
};

pub(crate) fn input_highlighter_factory() -> InputHighlighterFactory {
    Rc::new(|language| {
        LanguageRegistry::singleton().has_parser(language).then(|| {
            Box::new(TreeSitterInputHighlighter::new(language)) as Box<dyn InputHighlighter>
        })
    })
}

const SYNC_PARSE_TIMEOUT: Duration = Duration::from_millis(2);
const SYNC_PARSE_MAX_BYTES: usize = 256 * 1024;
const PARSE_DEBOUNCE: Duration = Duration::from_millis(150);
/// The first parse of a large text lands a tree for about this many leading
/// bytes before the full parse, so the top of the document colors early.
const PREFIX_PARSE_BYTES: usize = 256 * 1024;
/// Above this size injected languages are not parsed.
const MAX_INJECTION_TEXT_BYTES: usize = 4 * 1024 * 1024;

struct TreeSitterInputHighlighter {
    inner: Rc<RefCell<SyntaxHighlighter>>,
    parse_task: Rc<RefCell<Option<Task<()>>>>,
    /// Whether a complete parse has landed; until then parses start at once.
    parsed: Rc<Cell<bool>>,
    /// Stamped from [`next_generation`] whenever a parse is installed.
    generation: Rc<Cell<u64>>,
}

/// A process-wide unique stamp, so a replaced highlighter never repeats the
/// generation of the one before it.
fn next_generation() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

impl TreeSitterInputHighlighter {
    fn new(language: &str) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SyntaxHighlighter::new(language))),
            parse_task: Rc::new(RefCell::new(None)),
            parsed: Rc::new(Cell::new(false)),
            generation: Rc::new(Cell::new(0)),
        }
    }

    /// Settle after the edits reached the tree: drop a pending background
    /// parse when the synchronous one `completed`, or schedule one for `text`.
    fn finish_update(
        &mut self,
        completed: bool,
        text: &Rope,
        folding: bool,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<EditorState>,
    ) {
        if completed {
            self.parsed.set(true);
            self.generation.set(next_generation());
            self.parse_task.borrow_mut().take();
            return;
        }

        let highlighter = self.inner.clone();
        let parse_task = self.parse_task.clone();
        let parsed = self.parsed.clone();
        let generation = self.generation.clone();
        let debounce = parsed.get();
        let language = highlighter.borrow().language().clone();
        let old_tree = highlighter.borrow().tree().cloned();
        let prefix = if old_tree.is_none() && !debounce {
            prefix_edit(text)
        } else {
            None
        };
        let injection_data = if text.len() <= MAX_INJECTION_TEXT_BYTES {
            highlighter.borrow().injection_parse_data()
        } else {
            None
        };
        let text = text.clone();
        let text_for_apply = text.clone();
        let cancel = Arc::new(AtomicBool::new(false));

        let task = cx.spawn_in(window, async move |entity, cx| {
            struct CancelOnDrop(Arc<AtomicBool>);
            impl Drop for CancelOnDrop {
                fn drop(&mut self) {
                    self.0.store(true, Ordering::Relaxed);
                }
            }
            let _cancel_guard = CancelOnDrop(cancel.clone());
            if debounce {
                cx.background_executor().timer(PARSE_DEBOUNCE).await;
            }

            let mut old_tree = old_tree;
            if let Some(edit) = prefix {
                let language = language.clone();
                let text = text.clone();
                let cancel = cancel.clone();
                let tree = cx
                    .background_executor()
                    .spawn(async move {
                        let mut tree =
                            parse_until(&language, &text, edit.start_byte, None, &cancel)?;
                        // Grow the prefix tree into an edited tree of the
                        // whole text, so it is reused like any edited tree.
                        tree.edit(&edit);
                        Some(tree)
                    })
                    .await;
                if let Some(tree) = tree {
                    if highlighter.borrow_mut().apply_background_tree(
                        tree.clone(),
                        &text_for_apply,
                        Vec::new(),
                    ) {
                        generation.set(next_generation());
                    }
                    if entity.update(cx, |_, cx| cx.notify()).is_err() {
                        return;
                    }
                    old_tree = Some(tree);
                }
            }

            let parse_cancel = cancel.clone();
            let result = cx
                .background_executor()
                .spawn(async move {
                    let tree = parse_until(
                        &language,
                        &text,
                        text.len(),
                        old_tree.as_ref(),
                        &parse_cancel,
                    )?;
                    let injections = injection_data.map_or_else(Default::default, |data| {
                        SyntaxHighlighter::compute_injection_layers(data, &tree, &text)
                    });
                    let folds = if folding {
                        extract_fold_ranges(&tree)
                    } else {
                        Vec::new()
                    };
                    Some((tree, injections, folds))
                })
                .await;

            if let Some((tree, injections, folds)) = result {
                if highlighter
                    .borrow_mut()
                    .apply_background_tree(tree, &text_for_apply, injections)
                {
                    parsed.set(true);
                    generation.set(next_generation());
                }
                let _ = entity.update(cx, |state, cx| {
                    state.apply_highlighter_fold_candidates(folds, cx);
                });
            }
        });
        parse_task.borrow_mut().replace(task);
    }
}

/// The edit that grows a parse of the leading lines of `text` into a parse of
/// all of it, or `None` when `text` is too small to be worth parsing in two
/// steps. The prefix ends at a line start.
fn prefix_edit(text: &Rope) -> Option<InputEdit> {
    if text.len() <= SYNC_PARSE_MAX_BYTES {
        return None;
    }
    let row = text.offset_to_point(PREFIX_PARSE_BYTES).row + 1;
    let start_byte = text.line_start_offset(row);
    if start_byte == 0 || start_byte.saturating_mul(2) > text.len() {
        return None;
    }
    let end = text.offset_to_point(text.len());
    Some(InputEdit {
        start_byte,
        old_end_byte: start_byte,
        new_end_byte: text.len(),
        start_position: Point::new(row, 0),
        old_end_position: Point::new(row, 0),
        new_end_position: Point::new(end.row, end.column),
    })
}

/// Parse the first `end` bytes of `text` on a fresh parser for `language`,
/// giving up once `cancel` is set.
fn parse_until(
    language: &str,
    text: &Rope,
    end: usize,
    old_tree: Option<&Tree>,
    cancel: &AtomicBool,
) -> Option<Tree> {
    let (mut parser, grammar) = LanguageRegistry::singleton().parser(language).ok()?;
    parser.set_language(&grammar).ok()?;
    let mut progress = |_: &tree_sitter::ParseState| {
        if cancel.load(Ordering::Relaxed) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
    let options = ParseOptions::new().progress_callback(&mut progress);
    let tree = parser.parse_with_options(
        &mut |offset, _| parse_input_bytes_until(text, offset, end),
        old_tree,
        Some(options),
    )?;
    (!cancel.load(Ordering::Relaxed)).then_some(tree)
}

fn parse_input_bytes_until(text: &Rope, offset: usize, end: usize) -> &[u8] {
    if offset >= end {
        return &[];
    }
    let bytes = parse_input_bytes(text, offset);
    &bytes[..bytes.len().min(end - offset)]
}

impl InputHighlightSnapshot for SyntaxSnapshot {
    fn text(&self) -> &Rope {
        SyntaxSnapshot::text(self)
    }

    fn styles(
        &self,
        range: &Range<usize>,
        resolver: &dyn HighlightStyleResolver,
    ) -> Vec<(Range<usize>, HighlightStyle)> {
        SyntaxSnapshot::styles(self, range, resolver)
    }
}

impl SyntaxHighlighter {
    pub(crate) fn update_input(
        &mut self,
        edit: Option<BaseInputEdit>,
        text: &Rope,
        timeout: Option<Duration>,
    ) -> bool {
        self.update(edit.map(to_tree_sitter_edit), text, timeout)
    }
}

impl InputHighlighter for TreeSitterInputHighlighter {
    fn language(&self) -> SharedString {
        self.inner.borrow().language().clone()
    }

    fn update(
        &mut self,
        edit: Option<BaseInputEdit>,
        text: &Rope,
        folding: bool,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<EditorState>,
    ) {
        let edit = edit.map(to_tree_sitter_edit);
        let completed = {
            let mut highlighter = self.inner.borrow_mut();
            if text.len() > SYNC_PARSE_MAX_BYTES {
                highlighter.edit_tree(edit, text);
                false
            } else {
                highlighter.update(edit, text, Some(SYNC_PARSE_TIMEOUT))
            }
        };
        self.finish_update(completed, text, folding, window, cx);
    }

    fn update_batch(
        &mut self,
        edits: &[(BaseInputEdit, Rope)],
        folding: bool,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<EditorState>,
    ) {
        let Some((_, text)) = edits.last() else {
            return;
        };
        let edits: Vec<InputEdit> = edits
            .iter()
            .map(|(edit, _)| to_tree_sitter_edit(*edit))
            .collect();
        let completed = {
            let mut highlighter = self.inner.borrow_mut();
            if text.len() > SYNC_PARSE_MAX_BYTES {
                for edit in edits {
                    highlighter.edit_tree(Some(edit), text);
                }
                false
            } else {
                highlighter.update_edits(&edits, text, Some(SYNC_PARSE_TIMEOUT))
            }
        };
        self.finish_update(completed, text, folding, window, cx);
    }

    fn styles(
        &self,
        range: &Range<usize>,
        resolver: &dyn HighlightStyleResolver,
    ) -> Vec<(Range<usize>, HighlightStyle)> {
        self.inner.borrow().styles(range, resolver)
    }

    fn snapshot(&self) -> Option<Arc<dyn InputHighlightSnapshot>> {
        let snapshot = self.inner.borrow().snapshot()?;
        Some(Arc::new(snapshot))
    }

    fn generation(&self) -> u64 {
        self.generation.get()
    }

    fn fold_ranges(&self, _: &Rope) -> Vec<FoldRange> {
        self.inner
            .borrow()
            .tree()
            .map(extract_fold_ranges)
            .unwrap_or_default()
    }

    fn fold_ranges_for_edit(&self, range: Range<usize>, _: &Rope) -> Vec<FoldRange> {
        self.inner
            .borrow()
            .tree()
            .map(|tree| extract_fold_ranges_in_range(tree, range))
            .unwrap_or_default()
    }
}

fn to_tree_sitter_edit(edit: BaseInputEdit) -> InputEdit {
    InputEdit {
        start_byte: edit.start_byte,
        old_end_byte: edit.old_end_byte,
        new_end_byte: edit.new_end_byte,
        start_position: Point::new(edit.start_position.row, edit.start_position.column),
        old_end_position: Point::new(edit.old_end_position.row, edit.old_end_position.column),
        new_end_position: Point::new(edit.new_end_position.row, edit.new_end_position.column),
    }
}

fn extract_fold_ranges(tree: &tree_sitter::Tree) -> Vec<FoldRange> {
    extract_fold_ranges_in_range(tree, 0..usize::MAX)
}

fn extract_fold_ranges_in_range(
    tree: &tree_sitter::Tree,
    byte_range: Range<usize>,
) -> Vec<FoldRange> {
    let mut ranges = Vec::new();
    collect_fold_ranges_in_children(tree.root_node(), &byte_range, &mut ranges);
    ranges.sort_by_key(|range| range.start_line);
    ranges.dedup_by_key(|range| range.start_line);
    ranges
}

fn collect_fold_ranges_in_children(
    node: tree_sitter::Node,
    bytes: &Range<usize>,
    ranges: &mut Vec<FoldRange>,
) {
    let mut cursor = node.walk();
    if cursor.goto_first_child_for_byte(bytes.start).is_none() {
        return;
    }
    loop {
        let child = cursor.node();
        if child.start_byte() >= bytes.end {
            return;
        }
        if child.is_named() {
            collect_fold_ranges(child, bytes, ranges);
        }
        if !cursor.goto_next_sibling() {
            return;
        }
    }
}

fn collect_fold_ranges(node: tree_sitter::Node, bytes: &Range<usize>, ranges: &mut Vec<FoldRange>) {
    if node.end_byte() <= bytes.start || node.start_byte() >= bytes.end {
        return;
    }
    let start = node.start_position().row;
    let end = node.end_position().row;
    if end.saturating_sub(start) < 2 {
        return;
    }
    ranges.push(FoldRange::new(start, end));
    collect_fold_ranges_in_children(node, bytes, ranges);
}
