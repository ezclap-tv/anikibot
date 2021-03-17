use chrono::{Date, DateTime, Utc};

use super::tz::TimeZone;

#[derive(Clone, Copy, PartialEq)]
pub enum StatsInterval {
    Year,
    Month,
    Week,
    Day,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatsSettings {
    /// An interval to retrieve the stats for.
    pub(crate) interval: StatsInterval,
    /// ISO 8601 date value.
    pub(crate) date: Date<Utc>,
    /// A timezone index.
    pub(crate) timezone: TimeZone,
}

impl Default for StatsSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsSettings {
    /// Creates a new `StatsSettings` object configured to fetch
    /// the stats for the past week with the current date (UTC) in the ISO 8601 format.
    pub fn new() -> Self {
        Self {
            interval: StatsInterval::Year,
            date: Utc::now().date(),
            timezone: TimeZone::UTC,
        }
    }

    /// Updates the field `interval` to the given interval value.
    pub fn interval(self, interval: StatsInterval) -> Self {
        Self { interval, ..self }
    }

    /// Updates the field `timezone` to the given timezone value.
    pub fn timezone(self, timezone: TimeZone) -> Self {
        Self { timezone, ..self }
    }

    /// Updates the field `date` to the given [`Date`] value.
    pub fn date(self, date: Date<Utc>) -> Self {
        Self { date, ..self }
    }

    /// Updates the field `date` to the date part of the given [`DateTime`] value.
    pub fn date_from_datetime(self, date: DateTime<Utc>) -> Self {
        Self {
            date: date.date(),
            ..self
        }
    }
}

impl std::fmt::Debug for StatsInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StatsInterval::Year => "year",
                StatsInterval::Month => "month",
                StatsInterval::Week => "week",
                StatsInterval::Day => "day",
            }
        )
    }
}

impl ToString for StatsInterval {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
