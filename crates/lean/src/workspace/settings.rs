use crate::explorer;
use crate::settings::Setting;

pub(crate) const RESTORE_SESSION: Setting<bool> = Setting::new("startup.restore_session", false);
pub(crate) const RESTORE_LAYOUT: Setting<bool> = Setting::new("workbench.restore_layout", false);
pub(crate) const DEFAULT_PANEL: Setting<String> =
    Setting::new_with("workbench.default_panel", default_panel);
pub(crate) const SIDEBAR_WIDTH: Setting<f32> = Setting::new("workbench.sidebar.width", 260.);
pub(crate) const STATUS_BAR_VISIBLE: Setting<bool> =
    Setting::new("workbench.status_bar.visible", true);
pub(crate) const TITLE_BAR_SEARCH_BOX: Setting<bool> =
    Setting::new("workbench.title_bar.search_box", true);

pub(crate) const SIDEBAR_WIDTH_RANGE: (f32, f32) = (170., 600.);

fn default_panel() -> String {
    explorer::PANEL.id.into()
}
