use std::collections::HashMap;
use std::time::{Duration, Instant};

use futures::StreamExt as _;
use futures::channel::mpsc::UnboundedReceiver;
use gpui_kit::*;

use super::Notebook;
use super::cell::{CellId, RunState};
use super::document::CellKind;
use super::kernel::{Kernel, KernelEvent, KernelSpec, Reply, kernel_program};
use super::settings::{INTERPRETER, STARTUP_TIMEOUT_S, STOP_ON_ERROR};
use crate::plugins::FileViewEvent;
use crate::toolchain::ToolchainsEvent;

const RUNNABLE_LANGUAGE: &str = "python";

pub(super) struct Session {
    kernel: Kernel,
    environment: HashMap<String, String>,
    interpreter: String,
    pub(super) outdated: bool,
    pub(super) ready: bool,
    pub(super) running: Option<Running>,
    _events_task: Task<()>,
}

pub(super) struct Running {
    pub(super) cell: CellId,
    request: String,
    started: Instant,
}

impl Notebook {
    pub(super) fn run_cells(&mut self, ids: Vec<CellId>, cx: &mut Context<Self>) {
        for id in ids {
            let Some(cell) = self.cell_mut(id) else {
                continue;
            };
            match cell.kind {
                CellKind::Markdown => cell.editing = false,
                CellKind::Code if !cell.is_pending() => {
                    cell.run_state = RunState::Queued;
                    self.queue.push_back(id);
                }
                CellKind::Code | CellKind::Raw => {}
            }
        }
        if !self.queue.is_empty() {
            self.ensure_session(cx);
        }
        self.pump(cx);
        cx.emit(FileViewEvent::StatusChanged);
        cx.notify();
    }

    pub(super) fn run_all(&mut self, cx: &mut Context<Self>) {
        let ids = self.cells.iter().map(|cell| cell.id).collect();
        self.run_cells(ids, cx);
    }

    pub(super) fn interrupt_kernel(&mut self, cx: &mut Context<Self>) {
        if let Some(session) = &self.session
            && session.running.is_some()
        {
            session.kernel.interrupt();
        }
        self.cancel_queue();
        cx.notify();
    }

    pub(super) fn restart_kernel(&mut self, cx: &mut Context<Self>) {
        self.stop_session();
        self.ensure_session(cx);
        cx.emit(FileViewEvent::StatusChanged);
        cx.notify();
    }

    pub(super) fn handle_toolchains_event(
        &mut self,
        event: &ToolchainsEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, ToolchainsEvent::Discovered) && self.awaiting_toolchains {
            self.awaiting_toolchains = false;
            self.ensure_session(cx);
            self.pump(cx);
            cx.emit(FileViewEvent::StatusChanged);
        }
        self.refresh_outdated(cx);
    }

    pub(super) fn refresh_outdated(&mut self, cx: &mut Context<Self>) {
        if self.session.is_some() {
            let environment = self.toolchains.read(cx).environment();
            let interpreter = INTERPRETER.get(cx);
            if let Some(session) = &mut self.session {
                session.outdated =
                    session.environment != environment || session.interpreter != interpreter;
            }
        }
        cx.notify();
    }

    pub(super) fn kill_kernel(&mut self) -> impl Future<Output = ()> + use<> {
        let kernel = self.session.take().map(|session| session.kernel);
        async move {
            if let Some(kernel) = kernel {
                kernel.kill().await;
            }
        }
    }

    fn ensure_session(&mut self, cx: &mut Context<Self>) {
        if self.session.is_some() {
            return;
        }
        if self.language != RUNNABLE_LANGUAGE {
            self.kernel_error = Some(
                format!(
                    "Lean can only run Python notebooks, this one is {}",
                    self.language
                )
                .into(),
            );
            self.cancel_queue();
            return;
        }
        if !self.toolchains.read(cx).is_discovered() {
            self.awaiting_toolchains = true;
            return;
        }
        let environment = self.toolchains.read(cx).environment();
        let interpreter = INTERPRETER.get(cx);
        let spec = KernelSpec {
            folder: self.folder.clone(),
            environment: environment.clone(),
            program: kernel_program(&interpreter),
            startup_timeout: Duration::from_secs(STARTUP_TIMEOUT_S.get(cx).into()),
        };
        match Kernel::start(spec) {
            Ok((kernel, events)) => {
                self.kernel_error = None;
                self.session = Some(Session {
                    kernel,
                    environment,
                    interpreter,
                    outdated: false,
                    ready: false,
                    running: None,
                    _events_task: Self::listen(events, cx),
                });
            }
            Err(error) => {
                log::error!("failed to start a notebook kernel: {error:#}");
                self.kernel_error = Some(format!("{error:#}").into());
                self.cancel_queue();
            }
        }
    }

    fn listen(mut events: UnboundedReceiver<KernelEvent>, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |notebook, cx| {
            while let Some(event) = events.next().await {
                let stopped = matches!(event, KernelEvent::Stopped(_));
                if let Err(error) =
                    notebook.update(cx, |notebook, cx| notebook.handle_kernel_event(event, cx))
                {
                    log::debug!("notebook dropped while its kernel was running: {error:#}");
                    break;
                }
                if stopped {
                    break;
                }
            }
        })
    }

    fn handle_kernel_event(&mut self, event: KernelEvent, cx: &mut Context<Self>) {
        match event {
            KernelEvent::Ready => {
                if let Some(session) = &mut self.session {
                    session.ready = true;
                }
                self.pump(cx);
            }
            KernelEvent::Reply { request, reply } => self.apply_reply(&request, reply, cx),
            KernelEvent::Stopped(error) => {
                if let Some(error) = &error {
                    log::warn!("notebook kernel stopped: {error}");
                }
                self.stop_session();
                self.kernel_error = error.map(SharedString::from);
            }
        }
        cx.emit(FileViewEvent::StatusChanged);
        cx.notify();
    }

    fn apply_reply(&mut self, request: &str, reply: Reply, cx: &mut Context<Self>) {
        let Some(session) = &mut self.session else {
            return;
        };
        let Some(running) = session
            .running
            .as_ref()
            .filter(|running| running.request == request)
        else {
            return;
        };
        let (id, started) = (running.cell, running.started);
        if matches!(reply, Reply::Idle) {
            session.running = None;
        }
        let Some(cell) = self.cell_mut(id) else {
            self.pump(cx);
            return;
        };
        match reply {
            Reply::Output(json) => cell.push_output(json, cx),
            Reply::Clear { wait } => cell.clear_outputs(wait),
            Reply::ExecutionCount(count) => cell.execution_count = Some(count),
            Reply::Idle => {
                cell.finish_run(started);
                if cell.has_error() && STOP_ON_ERROR.get(cx) {
                    self.cancel_queue();
                }
                self.pump(cx);
            }
        }
        self.remeasure_cell(id);
        cx.emit(FileViewEvent::Edited);
    }

    fn pump(&mut self, cx: &mut Context<Self>) {
        let Some(session) = &self.session else {
            return;
        };
        if !session.ready || session.running.is_some() {
            return;
        }
        while let Some(id) = self.queue.pop_front() {
            let Some(cell) = self.cell_mut(id) else {
                continue;
            };
            let code = cell.editor.read(cx).value().to_string();
            cell.start_run();
            self.remeasure_cell(id);
            let Some(session) = &mut self.session else {
                return;
            };
            let request = session.kernel.execute(code);
            session.running = Some(Running {
                cell: id,
                request,
                started: Instant::now(),
            });
            return;
        }
    }

    fn cancel_queue(&mut self) {
        for id in std::mem::take(&mut self.queue) {
            if let Some(cell) = self.cell_mut(id) {
                cell.run_state = RunState::Idle;
            }
        }
    }

    fn stop_session(&mut self) {
        if let Some(running) = self.session.take().and_then(|session| session.running)
            && let Some(cell) = self.cell_mut(running.cell)
        {
            cell.run_state = RunState::Idle;
        }
        self.cancel_queue();
    }
}
