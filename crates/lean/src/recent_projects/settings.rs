use crate::settings::Setting;

pub(crate) const RECENT_LIMIT: Setting<usize> = Setting::new("recent_projects.limit", 20);
pub(crate) const MENU_LIMIT: Setting<usize> = Setting::new("recent_projects.menu_limit", 10);
