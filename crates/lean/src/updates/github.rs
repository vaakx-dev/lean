use anyhow::{Context as _, Result};
use serde::Deserialize;

use super::archive;
use super::build::short_commit;
use super::settings::Channel;

const API: &str = "https://api.github.com/repos";
const USER_AGENT: &str = concat!("lean/", env!("CARGO_PKG_VERSION"));
const CHECKSUMS: &str = "SHA256SUMS";
const DOWNLOAD_LIMIT: u64 = 512 * 1024 * 1024;

#[derive(Clone)]
pub(super) struct Release {
    pub(super) channel: Channel,
    pub(super) tag: String,
    pub(super) commit: String,
    pub(super) archive: Asset,
    pub(super) checksums: Asset,
}

impl Release {
    pub(super) fn label(&self) -> String {
        match self.channel {
            Channel::Stable => self.tag.clone(),
            Channel::Nightly | Channel::Dev => {
                format!("{} {}", self.channel.name(), short_commit(&self.commit))
            }
        }
    }
}

#[derive(Clone, Deserialize)]
pub(super) struct Asset {
    pub(super) name: String,
    #[serde(rename = "browser_download_url")]
    url: String,
}

#[derive(Deserialize)]
struct ApiRelease {
    tag_name: String,
    target_commitish: String,
    assets: Vec<Asset>,
}

fn release_url(channel: Channel) -> String {
    let repo = env!("CARGO_PKG_REPOSITORY").trim_start_matches("https://github.com/");
    match channel {
        Channel::Stable => format!("{API}/{repo}/releases/latest"),
        Channel::Nightly | Channel::Dev => {
            format!("{API}/{repo}/releases/tags/{}", channel.name())
        }
    }
}

fn get(url: &str) -> Result<ureq::Body> {
    let response = ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("couldn't reach {url}"))?;
    Ok(response.into_body())
}

fn find_asset(release: &ApiRelease, name: &str) -> Result<Asset> {
    release
        .assets
        .iter()
        .find(|asset| asset.name == name)
        .cloned()
        .with_context(|| format!("the {} release has no {name}", release.tag_name))
}

pub(super) fn latest(channel: Channel) -> Result<Release> {
    let release: ApiRelease = get(&release_url(channel))?.read_json()?;
    let archive_name = archive::file_name().context("Lean has no builds for this platform")?;
    Ok(Release {
        channel,
        archive: find_asset(&release, &archive_name)?,
        checksums: find_asset(&release, CHECKSUMS)?,
        tag: release.tag_name,
        commit: release.target_commitish,
    })
}

pub(super) fn download(asset: &Asset) -> Result<Vec<u8>> {
    get(&asset.url)?
        .with_config()
        .limit(DOWNLOAD_LIMIT)
        .read_to_vec()
        .with_context(|| format!("couldn't download {}", asset.name))
}
