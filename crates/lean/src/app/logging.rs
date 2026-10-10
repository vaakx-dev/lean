use env_logger::Env;
use gpui_kit::App;
use log::LevelFilter;

use super::settings::{LOG_LEVEL, LogLevel};

const LOG_VARIABLE: &str = "RUST_LOG";

pub(crate) fn init_logging() {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    if !overridden_by_environment() {
        log::set_max_level(LevelFilter::Warn);
    }
}

pub(super) fn follow_log_level(cx: &mut App) {
    if overridden_by_environment() {
        return;
    }
    log::set_max_level(level_filter(LOG_LEVEL.get(cx)));
    LOG_LEVEL
        .observe_app(cx, |level, _| log::set_max_level(level_filter(level)))
        .detach();
}

fn overridden_by_environment() -> bool {
    std::env::var_os(LOG_VARIABLE).is_some()
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
