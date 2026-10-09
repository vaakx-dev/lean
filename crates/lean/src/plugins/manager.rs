use gpui_kit::component::switch::Switch;
use gpui_kit::component::{ActiveTheme as _, Icon, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::host::is_active;
use super::{PLUGINS, Plugin};
use crate::settings;

const DIALOG_WIDTH: Pixels = px(520.);

pub(crate) fn open_plugin_manager(window: &mut Window, cx: &mut App) {
    let list = cx.new(|_| PluginList);
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("Plugins")
            .width(DIALOG_WIDTH)
            .child(list.clone())
    });
}

struct PluginList;

impl Render for PluginList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let needs_restart = PLUGINS
            .iter()
            .any(|plugin| settings::plugin_enabled(plugin.id, cx) != is_active(plugin.id, cx));
        v_flex()
            .gap_4()
            .when(PLUGINS.is_empty(), |list| {
                list.child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("This build of Lean has no plugins."),
                )
            })
            .children(PLUGINS.iter().map(|plugin| plugin_row(plugin, cx)))
            .when(needs_restart, |list| {
                list.child(
                    h_flex()
                        .gap_2()
                        .text_sm()
                        .text_color(theme.warning)
                        .child(Icon::empty().path("icons/triangle-alert.svg").size_4())
                        .child("Restart Lean to apply plugin changes."),
                )
            })
    }
}

fn plugin_row(plugin: &'static Plugin, cx: &Context<PluginList>) -> Div {
    let id = plugin.id;
    h_flex()
        .justify_between()
        .gap_4()
        .child(
            v_flex()
                .min_w_0()
                .gap_0p5()
                .child(div().font_weight(FontWeight::SEMIBOLD).child(plugin.name))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(plugin.description),
                ),
        )
        .child(
            Switch::new(id)
                .checked(settings::plugin_enabled(id, cx))
                .on_click(cx.listener(move |_, enabled: &bool, _, cx| {
                    settings::set_plugin_enabled(id, *enabled, cx);
                    cx.notify();
                })),
        )
}
