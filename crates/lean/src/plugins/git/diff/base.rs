#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiffBase {
    Index,
    Staged,
}

impl DiffBase {
    pub(crate) fn key(self) -> &'static str {
        match self {
            DiffBase::Index => "index",
            DiffBase::Staged => "staged",
        }
    }

    pub(crate) fn title(self) -> &'static str {
        match self {
            DiffBase::Index => "Working Tree",
            DiffBase::Staged => "Index",
        }
    }

    pub(crate) fn comparison(self) -> &'static str {
        match self {
            DiffBase::Index => "Index ↔ Working Tree",
            DiffBase::Staged => "HEAD ↔ Index",
        }
    }
}
