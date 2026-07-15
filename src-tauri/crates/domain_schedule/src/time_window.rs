use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("时间必须是 HH:MM 格式，收到 {value}")]
pub struct TimeOfDayError {
    value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeOfDay(u16);

impl TimeOfDay {
    pub fn parse(value: &str) -> Result<Self, TimeOfDayError> {
        let bytes = value.as_bytes();
        let valid = bytes.len() == 5
            && bytes[2] == b':'
            && bytes
                .iter()
                .enumerate()
                .all(|(index, byte)| index == 2 || byte.is_ascii_digit());
        if !valid {
            return Err(TimeOfDayError {
                value: value.to_string(),
            });
        }

        let hour = (bytes[0] - b'0') * 10 + bytes[1] - b'0';
        let minute = (bytes[3] - b'0') * 10 + bytes[4] - b'0';
        Self::from_hour_minute(hour, minute).ok_or_else(|| TimeOfDayError {
            value: value.to_string(),
        })
    }

    pub const fn from_hour_minute(hour: u8, minute: u8) -> Option<Self> {
        if hour < 24 && minute < 60 {
            Some(Self(hour as u16 * 60 + minute as u16))
        } else {
            None
        }
    }

    pub const fn hour(self) -> u8 {
        (self.0 / 60) as u8
    }

    pub const fn minute(self) -> u8 {
        (self.0 % 60) as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeWindow {
    start: Option<TimeOfDay>,
    end: Option<TimeOfDay>,
}

impl TimeWindow {
    pub fn parse(start: Option<&str>, end: Option<&str>) -> Result<Self, TimeOfDayError> {
        Ok(Self {
            start: start.map(TimeOfDay::parse).transpose()?,
            end: end.map(TimeOfDay::parse).transpose()?,
        })
    }

    pub const fn is_unbounded(self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    pub const fn start(self) -> Option<TimeOfDay> {
        self.start
    }

    pub fn contains(self, time: TimeOfDay) -> bool {
        match (self.start, self.end) {
            (None, None) => true,
            (Some(start), None) => time >= start,
            (None, Some(end)) => time <= end,
            (Some(start), Some(end)) if start <= end => time >= start && time <= end,
            (Some(start), Some(end)) => time >= start || time <= end,
        }
    }

    pub fn starts_previous_day(self, time: TimeOfDay) -> bool {
        match (self.start, self.end) {
            (Some(start), Some(end)) if start > end => time <= end,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_an_overnight_window() {
        let window = TimeWindow::parse(Some("22:00"), Some("02:00")).unwrap();
        let after_midnight = TimeOfDay::parse("01:00").unwrap();

        assert!(window.contains(TimeOfDay::parse("23:00").unwrap()));
        assert!(window.contains(after_midnight));
        assert!(!window.contains(TimeOfDay::parse("12:00").unwrap()));
        assert!(window.starts_previous_day(after_midnight));
    }
}
