use std::fmt;
use std::io;

const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;

pub(in crate::editor) enum ReadError {
    NotFound,
    PermissionDenied,
    IsFolder,
    TooLarge { size: u64, limit: u64 },
    Io(io::Error),
}

impl From<io::Error> for ReadError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound,
            io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            io::ErrorKind::IsADirectory => Self::IsFolder,
            _ => Self::Io(error),
        }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "the file doesn't exist"),
            Self::PermissionDenied => write!(f, "permission denied"),
            Self::IsFolder => write!(f, "it is a folder"),
            Self::TooLarge { size, limit } => write!(
                f,
                "the file is {} MB, larger than the {} MB limit set in Settings › Files",
                size.div_ceil(BYTES_PER_MEGABYTE),
                limit / BYTES_PER_MEGABYTE
            ),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}
