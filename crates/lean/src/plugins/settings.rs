use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::group_box::GroupBoxVariant;
use gpui_kit::component::setting::{SettingGroup, SettingItem, SettingPage};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{ActiveTheme as _, Disableable as _, Icon, Sizable as _, h_flex};
use gpui_kit::*;

use super::host::is_active;
use super::{PLUGINS, Plugin};
use crate::app;
use crate::settings::ui::fields::{Field, group};
use crate::settings::{plugin_enabled, set_plugin_enabled};

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

fn restart_notice() -> SettingGroup {
    SettingGroup::new()
        .variant(GroupBoxVariant::Normal)
        .item(SettingItem::render(|_, _, cx| render_restart_notice(cx)).keywords(["restart"]))
}

fn render_restart_notice(cx: &App) -> Div {
    let theme = cx.theme();
    h_flex()
        .w_full()
        .gap_2()
        .px_3()
        .py_2()
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.warning)
        .bg(theme.warning.opacity(0.1))
        .child(
            Icon::empty()
                .path("icons/triangle-alert.svg")
                .size_4()
                .text_color(theme.warning),
        )
        .child(
            div()
                .flex_1()
                .text_sm()
                .child("Restart Lean to apply plugin changes"),
        )
        .child(
            Button::new("restart-lean")
                .primary()
                .small()
                .label("Restart")
                .on_click(|_, _, cx| app::restart(cx)),
        )
}
