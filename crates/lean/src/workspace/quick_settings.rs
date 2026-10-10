mod controls;
mod rows;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::popover::{Popover, PopoverState};
use gpui_kit::component::separator::Separator;
use gpui_kit::component::{ActiveTheme as _, Icon, v_flex};
use gpui_kit::*;

use super::Workspace;
use crate::app::actions::OpenSettings;
use crate::editor::EditorArea;
use crate::editor::settings::{MINIMAP_ENABLED, SOFT_WRAP};
use crate::settings;
use crate::theme::settings::{THEME, UI_FONT_SIZE};
use controls::{font_size_stepper, setting_switch, theme_select};
use rows::{heading, link_row, setting_row};

const WIDTH: Pixels = px(272.);
const TRIGGER_SIZE: Pixels = px(40.);
const TRIGGER_ICON_SIZE: Pixels = px(22.);

impl Workspace {
    pub(super) fn render_quick_settings(&self, cx: &Context<Self>) -> Popover {
        let editor_area = self.context.editor_area.clone();
        Popover::new("quick-settings")
            .anchor(Anchor::LeftCenter)
            .offset(px(6.))
            .overlay_closable(false)
            .w(WIDTH)
            .p_0()
            .trigger(
                Button::new("quick-settings-trigger")
                    .ghost()
                    .size(TRIGGER_SIZE)
                    .p_0()
                    .text_color(cx.theme().muted_foreground)
                    .tooltip_with_action("Settings", &OpenSettings, None)
                    .child(
                        Icon::empty()
                            .path("icons/settings.svg")
                            .size(TRIGGER_ICON_SIZE),
                    ),
            )
            .content(move |_, window, cx| render_content(&editor_area, window, cx))
    }
}

fn render_content(
    editor_area: &Entity<EditorArea>,
    window: &mut Window,
    cx: &mut Context<PopoverState>,
) -> Div {
    let popover = cx.entity();
    let theme_menu_open = window.use_keyed_state("quick-settings-theme-menu", cx, |_, _| false);
    let cx: &App = cx;
    let theme_picker = theme_select(theme_menu_open.clone(), cx);
    let font_size = font_size_stepper(cx);
    let minimap = setting_switch("quick-settings-minimap", MINIMAP_ENABLED, cx);
    let soft_wrap = setting_switch("quick-settings-soft-wrap", SOFT_WRAP, cx);
    let theme = cx.theme();
    v_flex()
        .p_1p5()
        .on_mouse_down_out({
            let popover = popover.clone();
            move |_, window, cx| {
                if !*theme_menu_open.read(cx) {
                    popover.update(cx, |popover, cx| popover.dismiss(window, cx));
                }
            }
        })
        .child(heading("Quick settings", theme))
        .child(setting_row(
            "Theme",
            THEME.is_modified(cx),
            theme_picker,
            theme,
        ))
        .child(setting_row(
            "UI font size",
            UI_FONT_SIZE.is_modified(cx),
            font_size,
            theme,
        ))
        .child(setting_row(
            "Show minimap",
            MINIMAP_ENABLED.is_modified(cx),
            minimap,
            theme,
        ))
        .child(setting_row(
            "Wrap long lines",
            SOFT_WRAP.is_modified(cx),
            soft_wrap,
            theme,
        ))
        .child(Separator::horizontal().my_1p5())
        .child(
            link_row(
                "quick-settings-open-all",
                "icons/settings.svg",
                "Open all settings",
                theme,
                close_then(&popover, editor_area, settings::open_settings),
            )
            .suffix(|window, _| {
                div().children(Kbd::global_binding_for_action(&OpenSettings, window))
            }),
        )
        .child(link_row(
            "quick-settings-keybindings",
            "icons/keyboard.svg",
            "Keybindings",
            theme,
            close_then(&popover, editor_area, settings::open_keybindings),
        ))
}

fn close_then(
    popover: &Entity<PopoverState>,
    editor_area: &Entity<EditorArea>,
    open: fn(&Entity<EditorArea>, &mut Window, &mut App),
) -> impl Fn(&mut Window, &mut App) + 'static {
    let popover = popover.clone();
    let editor_area = editor_area.clone();
    move |window, cx| {
        popover.update(cx, |popover, cx| popover.dismiss(window, cx));
        open(&editor_area, window, cx);
    }
}
