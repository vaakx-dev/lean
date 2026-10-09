use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gpui_kit::{AppContext as _, Context, EventEmitter, SharedString, Task};

use super::activation::{activation_command, activation_environment};
use super::discovery::discover;
use super::store::{self, Choices};
use super::{NO_TOOLCHAIN_LABEL, Toolchain};
use crate::terminal::{ShellKind, TerminalLaunch};

pub(crate) enum ToolchainsEvent {
    Discovered,
    Selected(SharedString),
}

pub(crate) struct Toolchains {
    project_root: PathBuf,
    available: Vec<Toolchain>,
    choices: Choices,
    discovered: bool,
    _discovery_task: Task<()>,
}

impl EventEmitter<ToolchainsEvent> for Toolchains {}

impl Toolchains {
    pub(crate) fn new(project_root: PathBuf, cx: &mut Context<Self>) -> Self {
        let mut toolchains = Self {
            project_root,
            available: Vec::new(),
            choices: Choices::new(),
            discovered: false,
            _discovery_task: Task::ready(()),
        };
        toolchains.refresh(cx);
        toolchains
    }

    pub(crate) fn is_discovered(&self) -> bool {
        self.discovered
    }

    pub(crate) fn refresh(&mut self, cx: &mut Context<Self>) {
        let root = self.project_root.clone();
        let load_choices = !self.is_discovered();
        let discovery = cx.background_spawn(async move {
            let choices = load_choices.then(|| store::load(&root));
            (discover(&root), choices)
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

    pub(crate) fn for_language<'a, 'l>(
        &'a self,
        language: &'l str,
    ) -> impl Iterator<Item = &'a Toolchain> {
        self.available
            .iter()
            .filter(move |toolchain| toolchain.language == language)
    }

    pub(crate) fn has_language(&self, language: &str) -> bool {
        self.for_language(language).next().is_some()
    }

    pub(crate) fn active(&self, language: &str) -> Option<&Toolchain> {
        match self.choices.get(language) {
            Some(None) => None,
            Some(Some(prefix)) => self
                .find(language, prefix)
                .or_else(|| self.default_for(language)),
            None => self.default_for(language),
        }
    }

    pub(crate) fn active_label(&self, language: &str) -> SharedString {
        self.active(language)
            .map(Toolchain::label)
            .unwrap_or_else(|| NO_TOOLCHAIN_LABEL.into())
    }

    pub(crate) fn select(
        &mut self,
        language: &str,
        prefix: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) {
        let previous = self.active_prefix(language);
        self.choices.insert(language.to_owned(), prefix);
        let root = self.project_root.clone();
        let choices = self.choices.clone();
        cx.background_spawn(async move { store::save(&root, &choices) })
            .detach();
        if self.active_prefix(language) != previous {
            cx.emit(ToolchainsEvent::Selected(self.active_label(language)));
        }
        cx.notify();
    }

    pub(crate) fn environment(&self) -> HashMap<String, String> {
        activation_environment(&self.available, &self.active_toolchains())
    }

    pub(crate) fn terminal_launch(&self, shell: ShellKind) -> TerminalLaunch {
        TerminalLaunch {
            environment: self.environment(),
            startup_command: activation_command(&self.active_toolchains(), shell),
        }
    }

    fn finish_discovery(
        &mut self,
        available: Vec<Toolchain>,
        choices: Option<Choices>,
        cx: &mut Context<Self>,
    ) {
        self.available = available;
        if let Some(choices) = choices {
            self.choices = choices;
        }
        self.discovered = true;
        cx.emit(ToolchainsEvent::Discovered);
        cx.notify();
    }

    fn active_prefix(&self, language: &str) -> Option<PathBuf> {
        self.active(language)
            .map(|toolchain| toolchain.prefix.clone())
    }

    fn active_toolchains(&self) -> Vec<&Toolchain> {
        let mut languages: Vec<&str> = self
            .available
            .iter()
            .map(|toolchain| toolchain.language)
            .collect();
        languages.sort_unstable();
        languages.dedup();
        languages
            .into_iter()
            .filter_map(|language| self.active(language))
            .collect()
    }

    fn find(&self, language: &str, prefix: &Path) -> Option<&Toolchain> {
        self.for_language(language)
            .find(|toolchain| toolchain.prefix == prefix)
    }

    fn default_for(&self, language: &str) -> Option<&Toolchain> {
        self.for_language(language)
            .find(|toolchain| toolchain.prefix.starts_with(&self.project_root))
            .or_else(|| {
                self.for_language(language)
                    .find(|toolchain| toolchain.is_inherited)
            })
    }
}
