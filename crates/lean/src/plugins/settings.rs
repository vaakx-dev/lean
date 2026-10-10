mod restart_notice;

use gpui_kit::component::Icon;
use gpui_kit::component::setting::SettingPage;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::{App, IntoElement as _, SharedString};

use super::host::is_active;
use super::{PLUGINS, Plugin};
use crate::settings::ui::fields::{Field, group};
use crate::settings::{plugin_enabled, set_plugin_enabled};
use restart_notice::restart_notice;

pub(crate) fn settings_page(cx: &App) -> SettingPage {
    let page = SettingPage::new("Plugins")
        .icon(Icon::empty().path("icons/puzzle.svg"))
        .description("Built-in features you can switch off.");
    let page = if needs_restart(cx) {
        page.group(restart_notice())
    } else {
        page
    };
    page.group(group("Installed", PLUGINS.iter().map(plugin_field)))
}

fn needs_restart(cx: &App) -> bool {
    PLUGINS
        .iter()
        .any(|plugin| plugin_enabled(plugin.id, cx) != is_active(plugin.id, cx))
}

fn plugin_field(plugin: &'static Plugin) -> Field {
    let id = plugin.id;
    Field::new(
        format!("plugins.{id}"),
        plugin.name,
        plugin.description,
        move |cx| !plugin_enabled(id, cx),
        move |cx| set_plugin_enabled(id, true, cx),
        move |options, _, cx| {
            Switch::new(SharedString::from(format!("plugin-{id}")))
                .checked(plugin_enabled(id, cx))
                .disabled(options.is_disabled())
                .with_size(options.size())
                .on_click(move |enabled: &bool, _, cx| set_plugin_enabled(id, *enabled, cx))
                .into_any_element()
        },
    )
    .keywords(["plugin", id])
}
