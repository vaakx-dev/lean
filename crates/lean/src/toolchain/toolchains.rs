mod active;
mod selection;
#[cfg(feature = "terminal")]
mod terminal_launch;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use gpui_kit::{AppContext as _, Context, EventEmitter, SharedString, Subscription, Task};

use super::Toolchain;
use super::activation::activation_environment;
use super::auto_refresh::{AutoRefresh, watched_folders};
use super::python::{DiscoveryOptions, discover};
use super::settings::{
    AUTO_REFRESH, DEFAULT_INTERPRETER, DefaultInterpreter, PYENV_ROOT, SEARCH_PATHS,
    VENV_SEARCH_DEPTH,
};
use super::store::{self, Choices};

const DISCOVERY_DELAY: Duration = Duration::from_secs(2);

pub(crate) enum ToolchainsEvent {
    Discovered,
    Selected(SharedString),
}

pub(crate) struct Toolchains {
    project_root: PathBuf,
    available: Vec<Toolchain>,
    active_labels: HashMap<&'static str, SharedString>,
    choices: Choices,
    default_interpreter: DefaultInterpreter,
    discovered: bool,
    discovering: bool,
    auto_refresh: Option<AutoRefresh>,
    _discovery_task: Task<()>,
    _settings: Vec<Subscription>,
}

impl EventEmitter<ToolchainsEvent> for Toolchains {}

impl Toolchains {
    pub(crate) fn new(project_root: PathBuf, cx: &mut Context<Self>) -> Self {
        Self {
            project_root,
            available: Vec::new(),
            active_labels: HashMap::new(),
            choices: Choices::new(),
            default_interpreter: DEFAULT_INTERPRETER.get(cx),
            discovered: false,
            discovering: false,
            auto_refresh: None,
            _discovery_task: Self::discover_when_idle(cx),
            _settings: Self::observe_settings(cx),
        }
    }

    fn observe_settings(cx: &mut Context<Self>) -> Vec<Subscription> {
        vec![
            DEFAULT_INTERPRETER.observe(cx, |toolchains, default, cx| {
                toolchains.set_default_interpreter(default, cx)
            }),
            VENV_SEARCH_DEPTH.observe(cx, |toolchains, _, cx| toolchains.rediscover(cx)),
            SEARCH_PATHS.observe(cx, |toolchains, _, cx| toolchains.rediscover(cx)),
            PYENV_ROOT.observe(cx, |toolchains, _, cx| toolchains.rediscover(cx)),
            AUTO_REFRESH.observe(cx, |toolchains, enabled, cx| {
                toolchains.set_auto_refresh(enabled, cx)
            }),
        ]
    }

    fn discover_when_idle(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |toolchains, cx| {
            cx.background_executor().timer(DISCOVERY_DELAY).await;
            toolchains
                .update(cx, |toolchains, cx| toolchains.ensure_discovered(cx))
                .ok();
        })
    }

    pub(crate) fn ensure_discovered(&mut self, cx: &mut Context<Self>) {
        if !self.discovered && !self.discovering {
            self.refresh(cx);
        }
    }

    #[cfg(feature = "notebook")]
    pub(crate) fn is_discovered(&self) -> bool {
        self.discovered
    }

    pub(crate) fn environment(&self) -> HashMap<String, String> {
        activation_environment(&self.available, &self.active_toolchains())
    }

    pub(crate) fn refresh(&mut self, cx: &mut Context<Self>) {
        let root = self.project_root.clone();
        let options = DiscoveryOptions::from_settings(&root, cx);
        let load_choices = !self.discovered;
        let watch = AUTO_REFRESH.get(cx);
        self.discovering = true;
        let discovery = cx.background_spawn(async move {
            let choices = load_choices.then(|| store::load(&root));
            let available = discover(&root, &options);
            let folders = watch.then(|| watched_folders(&root, &options.search_paths, &available));
            (available, choices, folders)
        });
        self._discovery_task = cx.spawn(async move |toolchains, cx| {
            let (available, choices, folders) = discovery.await;
            toolchains
                .update(cx, |toolchains, cx| {
                    toolchains.finish_discovery(available, choices, folders, cx)
                })
                .ok();
        });
    }

    fn rediscover(&mut self, cx: &mut Context<Self>) {
        if self.discovered || self.discovering {
            self.refresh(cx);
        }
    }

    fn set_auto_refresh(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if enabled {
            self.rediscover(cx);
        } else {
            self.auto_refresh = None;
        }
    }

    fn finish_discovery(
        &mut self,
        available: Vec<Toolchain>,
        choices: Option<Choices>,
        folders: Option<Vec<PathBuf>>,
        cx: &mut Context<Self>,
    ) {
        self.discovering = false;
        self.track_selection(cx, |toolchains| {
            toolchains.available = available;
            if let Some(mut loaded) = choices {
                loaded.append(&mut toolchains.choices);
                toolchains.choices = loaded;
            }
            toolchains.discovered = true;
        });
        self.sync_auto_refresh(folders, cx);
        cx.emit(ToolchainsEvent::Discovered);
        cx.notify();
    }

    fn sync_auto_refresh(&mut self, folders: Option<Vec<PathBuf>>, cx: &mut Context<Self>) {
        let enabled = AUTO_REFRESH.get(cx);
        let Some(folders) = folders.filter(|_| enabled) else {
            self.auto_refresh = None;
            return;
        };
        let known: HashSet<PathBuf> = self
            .available
            .iter()
            .map(|toolchain| toolchain.prefix.clone())
            .collect();
        if let Some(auto_refresh) = &self.auto_refresh
            && auto_refresh.watches(&folders)
        {
            auto_refresh.set_known(known);
            return;
        }
        self.auto_refresh = AutoRefresh::start(folders, known, cx);
    }
}
