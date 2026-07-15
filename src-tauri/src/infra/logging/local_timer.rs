use chrono::Local;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::FormatTime;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum LocalTimer {
    Time,
    TimeStamp,
    Day,
    DayStamp,
    DayTime,
}

impl FormatTime for LocalTimer {
    fn format_time(&self, writer: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        let format = match self {
            Self::Time => "%Y-%m-%d %H:%M:%S%",
            Self::TimeStamp => "%Y-%m-%d %H:%M:%S%.3f",
            Self::Day => "%Y-%m-%d",
            Self::DayStamp => "%m-%d %H:%M:%S%.3f",
            Self::DayTime => "%m-%d %H:%M:%S%",
        };
        write!(writer, "{}", Local::now().format(format))
    }
}
