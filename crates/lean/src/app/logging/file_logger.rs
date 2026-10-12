use std::fs::{self, File};
use std::io::Write as _;
use std::sync::Mutex;

use log::{Log, Metadata, Record};

use super::timestamp::utc_now;
use crate::storage::state_file;

const FILE_NAME: &str = "lean.log";
const PREVIOUS_FILE_NAME: &str = "lean.old.log";

struct FileLogger {
    file: Option<Mutex<File>>,
}

pub(super) fn install() {
    let logger: &'static FileLogger = Box::leak(Box::new(FileLogger { file: open() }));
    if log::set_logger(logger).is_err() {
        eprintln!("a logger was already installed");
    }
}

fn open() -> Option<Mutex<File>> {
    let path = state_file(FILE_NAME)?;
    fs::create_dir_all(path.parent()?).ok()?;
    let _ = fs::rename(&path, path.with_file_name(PREVIOUS_FILE_NAME));
    File::create(&path).ok().map(Mutex::new)
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{} {:<5} {}: {}\n",
            utc_now(),
            record.level(),
            record.target(),
            record.args()
        );
        if cfg!(debug_assertions) {
            eprint!("{line}");
        }
        if let Some(file) = &self.file
            && let Ok(mut file) = file.lock()
        {
            let _ = file.write_all(line.as_bytes());
        }
    }

    fn flush(&self) {
        if let Some(file) = &self.file
            && let Ok(mut file) = file.lock()
        {
            let _ = file.flush();
        }
    }
}
