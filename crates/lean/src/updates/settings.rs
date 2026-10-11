use gpui_kit::component::setting::SettingGroup;
use serde::{Deserialize, Serialize};

use super::build;
use crate::settings::Setting;
use crate::settings::ui::fields::{group, select, switch};

pub(super) const CHANNEL: Setting<Channel> = Setting::new_with("updates.channel", default_channel);
pub(super) const CHECK_AUTOMATICALLY: Setting<bool> =
    Setting::new("updates.check_automatically", true);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Channel {
    Stable,
    Nightly,
    Dev,
}

impl Channel {
    pub(super) const ALL: [Self; 3] = [Self::Stable, Self::Nightly, Self::Dev];

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Nightly => "nightly",
            Self::Dev => "dev",
        }
    }
}

fn default_channel() -> Channel {
    build::current().map_or(Channel::Stable, |build| build.channel)
}

pub(crate) fn settings_group() -> SettingGroup {
    group(
        "Updates",
        [
            select(
                &CHANNEL,
                "Update channel",
                "Stable gets tagged releases, nightly a daily build of main, dev every push to main.",
                [
                    (Channel::Stable, "Stable"),
                    (Channel::Nightly, "Nightly"),
                    (Channel::Dev, "Dev"),
                ],
            )
            .keywords(["release", "nightly", "dev", "version"]),
            switch(
                &CHECK_AUTOMATICALLY,
                "Check for updates automatically",
                "Look for a new version at startup and every 6 hours.",
            )
            .keywords(["update", "upgrade", "version"]),
        ],
    )
}
