use crate::infrastructure::core::{Deserialize, Serialize};
use chrono::Local;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::FormatTime;

#[derive(Debug, Serialize, Deserialize)]
pub enum LocalTimer {
    Time,
    TimeStamp,
    Day,
    DayStamp,
    DayTime,
}

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        match self {
            LocalTimer::Time => {
                write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S%"))
            }
            LocalTimer::TimeStamp => {
                write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
            }
            LocalTimer::Day => {
                write!(w, "{}", Local::now().format("%Y-%m-%d"))
            }
            LocalTimer::DayStamp => {
                write!(w, "{}", Local::now().format("%m-%d %H:%M:%S%.3f"))
            }
            LocalTimer::DayTime => {
                write!(w, "{}", Local::now().format("%m-%d %H:%M:%S%"))
            }
        }
    }
}
