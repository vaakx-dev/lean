use gpui_kit::App;
use gpui_kit::component::setting::SettingGroup;
use serde::{Deserialize, Serialize};

use crate::settings::Setting;
use crate::settings::ui::fields::{group, select, switch};

pub(crate) const DEFAULT_ZOOM: Setting<DefaultZoom> =
    Setting::new("media.image.default_zoom", DefaultZoom::Fit);
pub(crate) const UPSCALE_SMALL: Setting<bool> = Setting::new("media.image.upscale_small", false);
pub(crate) const WHEEL: Setting<WheelAction> =
    Setting::new("media.image.wheel", WheelAction::ZoomWithCtrl);
pub(crate) const BACKGROUND: Setting<ImageBackground> =
    Setting::new("media.image.background", ImageBackground::Checkerboard);
pub(crate) const IMAGE_TOOLBAR: Setting<bool> = Setting::new("media.image.toolbar", true);
pub(crate) const SVG_DEFAULT_MODE: Setting<SvgMode> =
    Setting::new("media.svg.default_mode", SvgMode::Split);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DefaultZoom {
    Fit,
    ActualSize,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WheelAction {
    ZoomWithCtrl,
    Zoom,
    Pan,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImageBackground {
    Checkerboard,
    Solid,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SvgMode {
    Code,
    Preview,
    Split,
}

pub(crate) fn settings_groups(_: &App) -> Vec<SettingGroup> {
    vec![
        group(
            "Images",
            [
                select(
                    &DEFAULT_ZOOM,
                    "Default zoom",
                    "How big an image is when it opens.",
                    [
                        (DefaultZoom::Fit, "Fit to view"),
                        (DefaultZoom::ActualSize, "Actual size"),
                    ],
                )
                .keywords(["scale", "100%"]),
                switch(
                    &UPSCALE_SMALL,
                    "Enlarge small images",
                    "Fit to view also scales images up beyond their actual size.",
                )
                .keywords(["upscale", "stretch", "zoom"]),
                select(
                    &WHEEL,
                    "Mouse wheel",
                    "What scrolling over an image does.",
                    [
                        (WheelAction::ZoomWithCtrl, "Zoom with Ctrl, otherwise pan"),
                        (WheelAction::Zoom, "Zoom"),
                        (WheelAction::Pan, "Pan"),
                    ],
                )
                .keywords(["scroll", "zoom", "pan"]),
                select(
                    &BACKGROUND,
                    "Background",
                    "What shows through transparent pixels.",
                    [
                        (ImageBackground::Checkerboard, "Checkerboard"),
                        (ImageBackground::Solid, "Solid"),
                    ],
                )
                .keywords(["transparency", "alpha", "checkerboard"]),
                switch(
                    &IMAGE_TOOLBAR,
                    "Show toolbar",
                    "Show zoom controls above images.",
                )
                .keywords(["zoom", "controls"]),
            ],
        ),
        group(
            "SVG",
            [select(
                &SVG_DEFAULT_MODE,
                "Default view",
                "What an SVG file shows when it opens.",
                [
                    (SvgMode::Code, "Code"),
                    (SvgMode::Preview, "Preview"),
                    (SvgMode::Split, "Code and preview"),
                ],
            )
            .keywords(["preview", "split", "vector"])],
        ),
    ]
}
