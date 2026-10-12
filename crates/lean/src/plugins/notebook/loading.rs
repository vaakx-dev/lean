use std::mem;
use std::sync::Arc;

use anyhow::Result;
use gpui_kit::*;
use serde_json::{Map, Value};

use super::Notebook;
use super::document::{CellData, Document};
use super::output::{Output, OutputDecoder, release_outputs};
use crate::editor::file_text::{LineEnding, decode_text};
use crate::plugins::FileViewEvent;

pub(super) enum LoadState {
    Loading,
    Failed(SharedString),
    Loaded(LineEnding),
}

struct Loaded {
    line_ending: LineEnding,
    language: String,
    fields: Map<String, Value>,
    cells: Vec<(CellData, Vec<Output>)>,
}

fn load(bytes: Vec<u8>, decoder: &OutputDecoder) -> Result<Loaded> {
    let (text, line_ending) = decode_text(bytes)?;
    let document = Document::parse(&text)?;
    let language = document.language();
    let cells = document
        .cells
        .into_iter()
        .map(|mut data| {
            let outputs = mem::take(&mut data.outputs)
                .into_iter()
                .map(|json| decoder.decode(json))
                .collect();
            (data, outputs)
        })
        .collect();
    Ok(Loaded {
        line_ending,
        language,
        fields: document.fields,
        cells,
    })
}

impl Notebook {
    pub(super) fn load_in_background(
        &mut self,
        bytes: Vec<u8>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let decoder = OutputDecoder::new(cx);
        self._loading = cx.spawn_in(window, async move |notebook, cx| {
            let loaded = cx
                .background_spawn(async move { load(bytes, &decoder) })
                .await;
            cx.update(|window, cx| {
                notebook
                    .update(cx, |notebook, cx| {
                        notebook.finish_loading(loaded, window, cx)
                    })
                    .ok();
            })
            .ok();
        });
    }

    pub(super) fn load_now(
        &mut self,
        bytes: Vec<u8>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let loaded = load(bytes, &OutputDecoder::new(cx))?;
        self._loading = Task::ready(());
        self.apply(loaded, window, cx);
        Ok(())
    }

    fn finish_loading(
        &mut self,
        loaded: Result<Loaded>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match loaded {
            Ok(loaded) => self.apply(loaded, window, cx),
            Err(error) => {
                log::warn!("failed to open a notebook: {error:#}");
                self.load_state = LoadState::Failed(format!("{error:#}").into());
                cx.emit(FileViewEvent::StatusChanged);
                cx.notify();
            }
        }
    }

    fn apply(&mut self, loaded: Loaded, window: &mut Window, cx: &mut Context<Self>) {
        let previous_rows = self.list_state.item_count();
        let previous = mem::take(&mut self.cells);
        release_outputs(previous.into_iter().flat_map(|cell| cell.outputs), cx);
        self.language = loaded.language;
        self.fields = Arc::new(loaded.fields);
        let cells = loaded
            .cells
            .into_iter()
            .map(|(data, outputs)| self.create_cell(data, outputs, window, cx))
            .collect();
        self.cells = cells;
        self.selected = self.cells.first().map(|cell| cell.id);
        self.splice_rows(0..previous_rows, 0..self.cells.len() + 1, cx);
        let saved = self.saved_cells(cx);
        self.mark_saved(saved, cx);
        self.load_state = LoadState::Loaded(loaded.line_ending);
        cx.emit(FileViewEvent::StatusChanged);
        cx.notify();
    }
}
