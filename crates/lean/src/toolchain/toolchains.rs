mod active;
mod launch;
mod preferences;
mod selection;

use std::path::PathBuf;

use gpui_kit::{AppContext as _, Context, EventEmitter, SharedString, Subscription, Task};

use super::Toolchain;
use super::auto_refresh::AutoRefresh;
use super::discovery::{DiscoveryOptions, discover};
use super::settings::{AUTO_REFRESH, DEFAULT_INTERPRETER, DefaultInterpreter};
use super::store::{self, Choices};

pub(crate) enum ToolchainsEvent {
    Discovered,
    Selected(SharedString),
    TerminalActivationChanged,
}

pub(crate) struct Toolchains {
    project_root: PathBuf,
    available: Vec<Toolchain>,
    choices: Choices,
    default_interpreter: DefaultInterpreter,
    discovered: bool,
    auto_refresh: Option<AutoRefresh>,
    _discovery_task: Task<()>,
    _settings: Vec<Subscription>,
}

impl EventEmitter<ToolchainsEvent> for Toolchains {}

impl Toolchains {
    pub(crate) fn new(project_root: PathBuf, cx: &mut Context<Self>) -> Self {
        let mut toolchains = Self {
            project_root,
            available: Vec::new(),
            choices: Choices::new(),
            default_interpreter: DEFAULT_INTERPRETER.get(cx),
            discovered: false,
            auto_refresh: None,
            _discovery_task: Task::ready(()),
            _settings: preferences::observe_settings(cx),
        };
        toolchains.refresh(cx);
        toolchains
    }

    pub(crate) fn is_discovered(&self) -> bool {
        self.discovered
    }

    pub(crate) fn refresh(&mut self, cx: &mut Context<Self>) {
        let root = self.project_root.clone();
        let options = DiscoveryOptions::from_settings(&root, cx);
        let load_choices = !self.is_discovered();
        let discovery = cx.background_spawn(async move {
            let choices = load_choices.then(|| store::load(&root));
            (discover(&root, &options), choices)
        });
        self._discovery_task = cx.spawn(async move |toolchains, cx| {
            let (available, choices) = discovery.await;
            if let Err(error) = toolchains.update(cx, |toolchains, cx| {
                toolchains.finish_discovery(available, choices, cx)
            }) {
                log::debug!("toolchains dropped before discovery finished: {error:#}");
            }
        });
    }

    pub(crate) fn refresh_if_automatic(&mut self, cx: &mut Context<Self>) {
        if AUTO_REFRESH.get(cx) {
            self.refresh(cx);
        }
    }

    fn set_auto_refresh(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if enabled {
            self.refresh(cx);
        } else {
            self.auto_refresh = None;
        }
    }

    fn finish_discovery(
        &mut self,
        available: Vec<Toolchain>,
        choices: Option<Choices>,
        cx: &mut Context<Self>,
    ) {
        let was_discovered = self.discovered;
        self.track_selection(was_discovered, cx, |toolchains| {
            toolchains.available = available;
            if let Some(mut loaded) = choices {
                loaded.append(&mut toolchains.choices);
                toolchains.choices = loaded;
            }
            toolchains.discovered = true;
        });
        self.sync_auto_refresh(cx);
        cx.emit(ToolchainsEvent::Discovered);
        cx.notify();
    }

    fn sync_auto_refresh(&mut self, cx: &mut Context<Self>) {
        self.auto_refresh = None;
        if !AUTO_REFRESH.get(cx) {
            return;
        }
        let options = DiscoveryOptions::from_settings(&self.project_root, cx);
        self.auto_refresh = AutoRefresh::start(
            &self.project_root,
            &options.search_paths,
            &self.available,
            cx,
        );
    }
}
