use std::path::{Path, PathBuf};

use gpui_kit::SharedString;

use super::Toolchains;
use crate::toolchain::settings::DefaultInterpreter;
use crate::toolchain::{NO_TOOLCHAIN_LABEL, Toolchain};

impl Toolchains {
    pub(crate) fn for_language<'a>(
        &'a self,
        language: &str,
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

    pub(super) fn active_prefix(&self, language: &str) -> Option<PathBuf> {
        self.active(language)
            .map(|toolchain| toolchain.prefix.clone())
    }

    pub(super) fn active_toolchains(&self) -> Vec<&Toolchain> {
        self.languages()
            .into_iter()
            .filter_map(|language| self.active(language))
            .collect()
    }

    pub(super) fn languages(&self) -> Vec<&'static str> {
        let mut languages: Vec<&'static str> = self
            .available
            .iter()
            .map(|toolchain| toolchain.language)
            .collect();
        languages.sort_unstable();
        languages.dedup();
        languages
    }

    fn find(&self, language: &str, prefix: &Path) -> Option<&Toolchain> {
        self.for_language(language)
            .find(|toolchain| toolchain.prefix == prefix)
    }

    fn default_for(&self, language: &str) -> Option<&Toolchain> {
        let inherited = || {
            self.for_language(language)
                .find(|toolchain| toolchain.is_inherited)
        };
        match self.default_interpreter {
            DefaultInterpreter::Project => self
                .for_language(language)
                .find(|toolchain| toolchain.prefix.starts_with(&self.project_root))
                .or_else(inherited),
            DefaultInterpreter::Inherited => inherited(),
            DefaultInterpreter::System => None,
        }
    }
}
