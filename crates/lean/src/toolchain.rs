mod activation;
mod discovery;
mod picker;
mod python;
mod shell_activation;
mod store;
mod toolchains;

use std::path::PathBuf;

use gpui_kit::SharedString;

use shell_activation::ShellActivation;

pub(crate) use picker::toolchain_picker;
pub(crate) use toolchains::{Toolchains, ToolchainsEvent};

const NO_TOOLCHAIN_LABEL: &str = "System";

pub(crate) struct Toolchain {
    pub(crate) language: &'static str,
    pub(crate) name: SharedString,
    pub(crate) kind: &'static str,
    pub(crate) version: Option<SharedString>,
    pub(crate) prefix: PathBuf,
    pub(crate) bin_dirs: Vec<PathBuf>,
    pub(crate) variables: Vec<(String, String)>,
    pub(crate) activation: Option<ShellActivation>,
    pub(crate) is_inherited: bool,
}

impl Toolchain {
    pub(crate) fn label(&self) -> SharedString {
        match &self.version {
            Some(version) => format!("{} {version} ({})", self.name, self.kind).into(),
            None => format!("{} ({})", self.name, self.kind).into(),
        }
    }
}
