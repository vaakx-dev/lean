use anyhow::Result;
use gpui_kit::SharedString;

const HISTORY_LIMIT: usize = 50;

#[derive(Clone)]
pub(crate) struct CommitSummary {
    pub(crate) id: git2::Oid,
    pub(crate) short_id: SharedString,
    pub(crate) parents: Vec<git2::Oid>,
    pub(crate) summary: SharedString,
    pub(crate) message: SharedString,
    pub(crate) author: SharedString,
    pub(crate) time: i64,
}

pub(super) fn read_history(repository: &git2::Repository) -> Result<Vec<CommitSummary>> {
    let mut revwalk = repository.revwalk()?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
    match revwalk.push_head() {
        Ok(()) => {}
        Err(error)
            if matches!(
                error.code(),
                git2::ErrorCode::UnbornBranch | git2::ErrorCode::NotFound
            ) =>
        {
            return Ok(Vec::new());
        }
        Err(error) => return Err(error.into()),
    }
    revwalk
        .take(HISTORY_LIMIT)
        .map(|id| read_commit(repository, id?))
        .collect()
}

fn read_commit(repository: &git2::Repository, id: git2::Oid) -> Result<CommitSummary> {
    let commit = repository.find_commit(id)?;
    let author = commit.author();
    let short_id: String = id.to_string().chars().take(7).collect();
    Ok(CommitSummary {
        id,
        short_id: short_id.into(),
        parents: commit.parent_ids().collect(),
        summary: commit.summary().unwrap_or_default().to_string().into(),
        message: String::from_utf8_lossy(commit.message_bytes())
            .trim_end()
            .to_string()
            .into(),
        author: author.name().unwrap_or("Unknown").to_string().into(),
        time: commit.time().seconds(),
    })
}
