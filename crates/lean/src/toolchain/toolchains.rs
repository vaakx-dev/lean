mod active;
mod selection;

use std::collections::HashMap;
use std::path::PathBuf;

use gpui_kit::{App, AppContext as _, Context, EventEmitter, SharedString, Subscription, Task};

use super::Toolchain;
use super::activation::{activation_command, activation_environment};
use super::auto_refresh;
use super::python::{DiscoveryOptions, discover};
use super::settings::{
    ACTIVATE_IN_TERMINAL, ACTIVATION_MODE, AUTO_REFRESH, ActivationMode, DEFAULT_INTERPRETER,
    DefaultInterpreter, PYENV_ROOT, SEARCH_PATHS, VENV_SEARCH_DEPTH,
};
use super::store::{self, Choices};
use crate::terminal::{ShellKind, TerminalLaunch};

pub(crate) enum ToolchainsEvent {
    Discovered,
    Selected(SharedString),
}

pub(crate) struct Toolchains {
    project_root: PathBuf,
    available: Vec<Toolchain>,
    choices: Choices,
    default_interpreter: DefaultInterpreter,
    discovered: bool,
    auto_refresh: Option<Task<()>>,
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
            _settings: Self::observe_settings(cx),
        };
        toolchains.refresh(cx);
        toolchains
    }

    fn observe_settings(cx: &mut Context<Self>) -> Vec<Subscription> {
        vec![
            DEFAULT_INTERPRETER.observe(cx, |toolchains, default, cx| {
                toolchains.set_default_interpreter(default, cx)
            }),
            VENV_SEARCH_DEPTH.observe(cx, |toolchains, _, cx| toolchains.refresh(cx)),
            SEARCH_PATHS.observe(cx, |toolchains, _, cx| toolchains.refresh(cx)),
            PYENV_ROOT.observe(cx, |toolchains, _, cx| toolchains.refresh(cx)),
            AUTO_REFRESH.observe(cx, |toolchains, enabled, cx| {
                toolchains.set_auto_refresh(enabled, cx)
            }),
        ]
    }

    pub(crate) fn is_discovered(&self) -> bool {
        self.discovered
    }

    pub(crate) fn environment(&self) -> HashMap<String, String> {
        activation_environment(&self.available, &self.active_toolchains())
    }

    pub(crate) fn terminal_launch(&self, shell: ShellKind, cx: &App) -> TerminalLaunch {
        if !ACTIVATE_IN_TERMINAL.get(cx) {
            return TerminalLaunch::default();
        }
        let startup_command = match ACTIVATION_MODE.get(cx) {
            ActivationMode::ActivateScript => activation_command(&self.active_toolchains(), shell),
            ActivationMode::EnvironmentOnly => None,
        };
        TerminalLaunch {
            environment: self.environment(),
            startup_command,
            ..TerminalLaunch::default()
        }
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
            toolchains
                .update(cx, |toolchains, cx| {
                    toolchains.finish_discovery(available, choices, cx)
                })
                .ok();
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
        self.track_selection(cx, |toolchains| {
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
        self.auto_refresh = auto_refresh::watch(
            &self.project_root,
            &options.search_paths,
            &self.available,
            cx,
        );
    }
}
