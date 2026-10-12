use gpui_kit::App;
use gpui_kit::component::Icon;
use gpui_kit::component::setting::{SettingGroup, SettingPage};
use serde::{Deserialize, Serialize};

use crate::settings::Setting;
use crate::settings::ui::fields::{Number, PathKind, group, list, number, path, select, switch};

pub(crate) const MAX_VENV_SEARCH_DEPTH: u32 = 4;

pub(crate) const DEFAULT_INTERPRETER: Setting<DefaultInterpreter> =
    Setting::new("python.default_interpreter", DefaultInterpreter::Project);
#[cfg(feature = "terminal")]
pub(crate) const ACTIVATE_IN_TERMINAL: Setting<bool> =
    Setting::new("python.activate_in_terminal", true);
#[cfg(feature = "terminal")]
pub(crate) const ACTIVATION_MODE: Setting<ActivationMode> =
    Setting::new("python.activation_mode", ActivationMode::ActivateScript);
pub(crate) const VENV_SEARCH_DEPTH: Setting<u32> = Setting::new("python.venv_search_depth", 1);
pub(crate) const SEARCH_PATHS: Setting<Vec<String>> =
    Setting::new("python.search_paths", Vec::new());
pub(crate) const PYENV_ROOT: Setting<Option<String>> = Setting::new("python.pyenv_root", None);
pub(crate) const AUTO_REFRESH: Setting<bool> = Setting::new("python.auto_refresh", false);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DefaultInterpreter {
    Project,
    Inherited,
    System,
}

#[cfg(feature = "terminal")]
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ActivationMode {
    ActivateScript,
    EnvironmentOnly,
}

pub(crate) fn settings_page(_: &App) -> SettingPage {
    let page = SettingPage::new("Toolchains")
        .icon(Icon::empty().path("icons/wrench.svg"))
        .description("Python interpreters and virtual environments.")
        .group(group(
            "Interpreter",
            [select(
                &DEFAULT_INTERPRETER,
                "Default interpreter",
                "Used when no interpreter was picked for this project.",
                [
                    (DefaultInterpreter::Project, "Project environment"),
                    (DefaultInterpreter::Inherited, "Inherited environment"),
                    (DefaultInterpreter::System, "System"),
                ],
            )
            .keywords(["python", "venv", "virtualenv", "conda", "default"])],
        ));
    #[cfg(feature = "terminal")]
    let page = page.group(terminal_group());
    page.group(discovery_group())
}

#[cfg(feature = "terminal")]
fn terminal_group() -> SettingGroup {
    group(
        "Terminal",
        [
            switch(
                &ACTIVATE_IN_TERMINAL,
                "Activate in terminal",
                "Activate the selected environment in new terminals.",
            )
            .keywords(["python", "venv", "shell"]),
            select(
                &ACTIVATION_MODE,
                "Activation mode",
                "Run the activate script, or only set PATH and environment variables.",
                [
                    (ActivationMode::ActivateScript, "Activate script"),
                    (ActivationMode::EnvironmentOnly, "Environment only"),
                ],
            )
            .keywords(["python", "venv", "conda activate", "shell"])
            .enabled_when(|cx| ACTIVATE_IN_TERMINAL.get(cx)),
        ],
    )
}

fn discovery_group() -> SettingGroup {
    group(
        "Discovery",
        [
            number(
                &VENV_SEARCH_DEPTH,
                "Virtual environment search depth",
                "How many folder levels below the project root to look for virtual environments.",
                Number::range(0., f64::from(MAX_VENV_SEARCH_DEPTH)).unit("levels"),
            )
            .keywords(["python", "venv", ".venv", "discover"]),
            list(
                &SEARCH_PATHS,
                "Extra search paths",
                "Comma-separated folders that contain environments. Relative paths start at the project root.",
                "~/.virtualenvs, ~/envs",
            )
            .keywords(["python", "venv", "conda", "discover", "folders"])
            .stacked(),
            path(
                &PYENV_ROOT,
                "pyenv root",
                "Empty uses $PYENV_ROOT, then ~/.pyenv (~/.pyenv/pyenv-win on Windows).",
                PathKind::Folder,
                "~/.pyenv",
            )
            .keywords(["python", "pyenv", "versions"]),
            switch(
                &AUTO_REFRESH,
                "Refresh automatically",
                "Look for environments again when an environment is added or removed.",
            )
            .keywords(["python", "rediscover", "watch", "pyvenv.cfg", "conda-meta"]),
        ],
    )
}
