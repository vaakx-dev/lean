use std::time::{SystemTime, UNIX_EPOCH};

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const WEEK: i64 = 7 * DAY;
const MONTH: i64 = 30 * DAY;
const YEAR: i64 = 365 * DAY;

pub(super) fn relative_time(timestamp: i64) -> String {
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(elapsed) => i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX),
        Err(error) => {
            log::warn!("the system clock is before the unix epoch: {error}");
            return String::new();
        }
    };
    let seconds = now.saturating_sub(timestamp).max(0);
    let (amount, unit) = match seconds {
        seconds if seconds < MINUTE => return "now".to_string(),
        seconds if seconds < HOUR => (seconds / MINUTE, "m"),
        seconds if seconds < DAY => (seconds / HOUR, "h"),
        seconds if seconds < WEEK => (seconds / DAY, "d"),
        seconds if seconds < MONTH => (seconds / WEEK, "w"),
        seconds if seconds < YEAR => (seconds / MONTH, "mo"),
        seconds => (seconds / YEAR, "y"),
    };
    format!("{amount}{unit}")
}
