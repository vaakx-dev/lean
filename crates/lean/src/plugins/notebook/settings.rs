use gpui_kit::App;
use gpui_kit::component::Icon;
use gpui_kit::component::setting::SettingPage;

use crate::settings::Setting;
use crate::settings::ui::fields::{Number, PathKind, group, number, path, switch};

pub(crate) const INTERPRETER: Setting<String> = Setting::new("notebook.interpreter", String::new());
pub(crate) const STARTUP_TIMEOUT_S: Setting<u32> = Setting::new("notebook.startup_timeout_s", 60);
pub(crate) const STOP_ON_ERROR: Setting<bool> = Setting::new("notebook.stop_on_error", true);
pub(crate) const OUTPUT_MAX_LINES: Setting<u32> = Setting::new("notebook.output_max_lines", 2000);
pub(crate) const SAVE_OUTPUTS: Setting<bool> = Setting::new("notebook.save_outputs", true);
pub(crate) const SHOW_RUN_TIME: Setting<bool> = Setting::new("notebook.show_run_time", true);
pub(crate) const LINE_NUMBERS: Setting<bool> = Setting::new("notebook.line_numbers", false);
pub(crate) const SOFT_WRAP: Setting<bool> = Setting::new("notebook.soft_wrap", false);

pub(crate) fn settings_page(_: &App) -> SettingPage {
    SettingPage::new("Notebook")
        .icon(Icon::empty().path("icons/notebook-text.svg"))
        .description("Kernel, outputs and cells of Jupyter notebooks.")
        .group(group(
            "Kernel",
            [
                path(
                    &INTERPRETER,
                    "Python interpreter",
                    "Runs the kernel. Empty uses python3 from the active toolchain.",
                    PathKind::File,
                    "python3",
                )
                .keywords(["python", "ipykernel", "venv", "toolchain"]),
                number(
                    &STARTUP_TIMEOUT_S,
                    "Startup timeout",
                    "How long to wait for a new kernel to answer.",
                    Number::range(5., 300.).unit("s"),
                )
                .keywords(["kernel", "start", "wait"]),
                switch(
                    &STOP_ON_ERROR,
                    "Stop on error",
                    "Skip the remaining queued cells when one fails.",
                )
                .keywords(["run all", "exception", "queue"]),
            ],
        ))
        .group(group(
            "Output",
            [
                number(
                    &OUTPUT_MAX_LINES,
                    "Output line limit",
                    "Longer text outputs are cut off.",
                    Number::range(100., 100000.).unit("lines"),
                )
                .keywords(["truncate", "stdout", "stderr", "traceback"]),
                switch(
                    &SAVE_OUTPUTS,
                    "Save outputs",
                    "Store outputs and execution counts in the .ipynb file.",
                )
                .keywords(["ipynb", "clear", "execution count"]),
                switch(
                    &SHOW_RUN_TIME,
                    "Show run time",
                    "Show how long each cell took to run.",
                )
                .keywords(["duration", "elapsed", "timing"]),
            ],
        ))
        .group(
            group(
                "Cells",
                [
                    switch(
                        &LINE_NUMBERS,
                        "Line numbers",
                        "Show line numbers in code cells.",
                    )
                    .keywords(["gutter"]),
                    switch(
                        &SOFT_WRAP,
                        "Wrap long lines",
                        "Wrap cell lines at the cell width.",
                    )
                    .keywords(["soft wrap", "word wrap"]),
                ],
            )
            .description("Tab size follows the editor setting."),
        )
}
