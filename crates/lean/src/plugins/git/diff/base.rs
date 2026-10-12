use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DiffBase {
    Index,
    Head,
}

impl DiffBase {
    pub(crate) fn key(self) -> &'static str {
        match self {
            DiffBase::Index => "index",
            DiffBase::Head => "head",
        }
    }

    pub(crate) fn title(self) -> &'static str {
        match self {
            DiffBase::Index => "Working Tree",
            DiffBase::Head => "Index",
        }
    }

    pub(crate) fn comparison(self) -> &'static str {
        match self {
            DiffBase::Index => "Index ↔ Working Tree",
            DiffBase::Head => "HEAD ↔ Index",
        }
    }
}
