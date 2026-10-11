use std::env::consts::{ARCH, EXE_SUFFIX, OS};
use std::io::Read as _;

use anyhow::{Context as _, Result};

const EXTENSION: &str = if cfg!(windows) { "zip" } else { "tar.gz" };

fn package_name() -> Option<String> {
    let target = match (OS, ARCH) {
        ("linux", "x86_64") => "linux-x86_64",
        ("windows", "x86_64") => "windows-x86_64",
        ("macos", "aarch64") => "macos-aarch64",
        _ => return None,
    };
    Some(format!("lean-{target}"))
}

pub(super) fn file_name() -> Option<String> {
    package_name().map(|package| format!("{package}.{EXTENSION}"))
}

pub(super) fn extract_program(archive: &[u8]) -> Result<Vec<u8>> {
    let package = package_name().context("Lean has no builds for this platform")?;
    read_entry(archive, &format!("{package}/lean{EXE_SUFFIX}"))
}

#[cfg(windows)]
fn read_entry(archive: &[u8], path: &str) -> Result<Vec<u8>> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(archive))?;
    let mut entry = zip
        .by_name(path)
        .with_context(|| format!("{path} is missing from the download"))?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[cfg(not(windows))]
fn read_entry(archive: &[u8], path: &str) -> Result<Vec<u8>> {
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(archive));
    for entry in tar.entries()? {
        let mut entry = entry?;
        if entry.path()? == std::path::Path::new(path) {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            return Ok(bytes);
        }
    }
    anyhow::bail!("{path} is missing from the download")
}
