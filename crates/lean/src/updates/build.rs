use semver::Version;

use super::github::Release;
use super::settings::Channel;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy)]
pub(super) struct Build {
    pub(super) channel: Channel,
    commit: &'static str,
}

impl Build {
    pub(super) fn is_superseded_by(&self, release: &Release) -> bool {
        if release.channel != self.channel {
            return true;
        }
        match self.channel {
            Channel::Stable => parse_version(&release.tag) > parse_version(VERSION),
            Channel::Nightly | Channel::Dev => release.commit != self.commit,
        }
    }
}

pub(super) fn current() -> Option<Build> {
    let stamp = option_env!("LEAN_CHANNEL")?;
    let channel = Channel::ALL
        .into_iter()
        .find(|channel| channel.name() == stamp)?;
    Some(Build {
        channel,
        commit: option_env!("LEAN_COMMIT").unwrap_or_default(),
    })
}

pub(crate) fn version_label() -> String {
    match current() {
        Some(build) if build.channel == Channel::Stable => VERSION.to_owned(),
        Some(build) => format!(
            "{VERSION} ({} {})",
            build.channel.name(),
            short_commit(build.commit)
        ),
        None => format!("{VERSION} (local build)"),
    }
}

pub(super) fn short_commit(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}

fn parse_version(tag: &str) -> Option<Version> {
    Version::parse(tag.trim_start_matches('v')).ok()
}
