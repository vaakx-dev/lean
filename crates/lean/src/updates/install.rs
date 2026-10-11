use std::path::PathBuf;

use anyhow::{Context as _, Result, ensure};
use sha2::{Digest as _, Sha256};

use super::archive;
use super::github::{self, Release};

fn verify(archive: &[u8], checksums: &str, name: &str) -> Result<()> {
    let expected = checksums
        .lines()
        .find_map(|line| {
            let (hash, file) = line.split_once(' ')?;
            (file.trim_start_matches([' ', '*']) == name).then_some(hash)
        })
        .with_context(|| format!("the checksums have no entry for {name}"))?;
    let actual = format!("{:x}", Sha256::digest(archive));
    ensure!(actual == expected, "the download of {name} is corrupt");
    Ok(())
}

fn replace_program(program: &[u8]) -> Result<()> {
    let staged = std::env::temp_dir().join(format!("lean-update-{}", uuid::Uuid::new_v4()));
    std::fs::write(&staged, program)?;
    let replaced = self_replace::self_replace(&staged);
    std::fs::remove_file(&staged).ok();
    replaced.context("couldn't replace the Lean program")
}

pub(super) fn run(release: &Release) -> Result<PathBuf> {
    let program = std::env::current_exe()?;
    let archive = github::download(&release.archive)?;
    let checksums = github::download(&release.checksums)?;
    verify(
        &archive,
        &String::from_utf8_lossy(&checksums),
        &release.archive.name,
    )?;
    replace_program(&archive::extract_program(&archive)?)?;
    Ok(program)
}
