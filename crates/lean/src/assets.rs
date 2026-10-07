use std::borrow::Cow;

use anyhow::Result;
use gpui_kit::{AssetSource, SharedString};

pub(crate) struct Assets;

#[derive(rust_embed::RustEmbed)]
#[folder = "../../assets"]
#[include = "images/**/*"]
#[include = "icons/**/*"]
#[include = "themes/**/*"]
struct LeanAssets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(file) = LeanAssets::get(path) {
            return Ok(Some(file.data));
        }
        gpui_kit::assets::AllAssets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths: Vec<SharedString> = LeanAssets::iter()
            .filter(|file| file.starts_with(path))
            .map(SharedString::from)
            .collect();
        paths.extend(gpui_kit::assets::AllAssets.list(path)?);
        Ok(paths)
    }
}

pub(crate) fn load_string(path: &str) -> Result<String> {
    let file = LeanAssets::get(path).ok_or_else(|| anyhow::anyhow!("missing asset {path}"))?;
    Ok(String::from_utf8(file.data.into_owned())?)
}
