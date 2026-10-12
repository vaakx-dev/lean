mod file_logger;
mod timestamp;

use gpui_kit::App;
use log::LevelFilter;

use super::settings::{LOG_LEVEL, LogLevel};

const LOG_VARIABLE: &str = "RUST_LOG";

pub(crate) fn init_logging() {
    file_logger::install();
    log::set_max_level(environment_level().unwrap_or(LevelFilter::Warn));
    log_panics();
}

pub(super) fn follow_log_level(cx: &mut App) {
    if environment_level().is_some() {
        return;
    }
    log::set_max_level(level_filter(LOG_LEVEL.get(cx)));
    LOG_LEVEL
        .observe_app(cx, |level, _| log::set_max_level(level_filter(level)))
        .detach();
}

fn environment_level() -> Option<LevelFilter> {
    let value = std::env::var(LOG_VARIABLE).ok()?;
    value
        .split(',')
        .filter_map(|directive| directive.rsplit('=').next())
        .filter_map(|level| level.trim().parse::<LevelFilter>().ok())
        .max()
}

fn log_panics() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("{info}");
        default_hook(info);
    }));
}

fn level_filter(level: LogLevel) -> LevelFilter {
    match level {
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Trace => LevelFilter::Trace,
    }
}
