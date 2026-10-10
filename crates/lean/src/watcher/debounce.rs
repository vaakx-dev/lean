use std::time::Duration;

use gpui_kit::App;

use crate::settings::Setting;

pub(crate) const DEBOUNCE_MS: Setting<u64> = Setting::new("watcher.debounce_ms", 150);

const SHORTEST_MS: u64 = 50;
const LONGEST_MS: u64 = 2000;

pub(super) fn debounce(cx: &App) -> Duration {
    Duration::from_millis(DEBOUNCE_MS.get(cx).clamp(SHORTEST_MS, LONGEST_MS))
}
