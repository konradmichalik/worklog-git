use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Period {
    Today,
    Yesterday,
    Hours(u32),
    Days(u32),
    Week,
}

#[derive(Debug)]
pub struct TimeRange {
    pub since: DateTime<Local>,
    pub until: Option<DateTime<Local>>,
}

impl FromStr for Period {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "today" => Ok(Period::Today),
            "yesterday" => Ok(Period::Yesterday),
            "week" => Ok(Period::Week),
            other => {
                if let Some(h) = other.strip_suffix('h') {
                    h.parse::<u32>()
                        .map(Period::Hours)
                        .map_err(|_| format!("Invalid hours: {other}"))
                } else if let Some(d) = other.strip_suffix('d') {
                    d.parse::<u32>()
                        .map(Period::Days)
                        .map_err(|_| format!("Invalid days: {other}"))
                } else {
                    Err(format!(
                        "Unknown period: {other}. Use: today, yesterday, 24h, 3d, 7d, week"
                    ))
                }
            }
        }
    }
}

impl Period {
    pub fn to_time_range(&self) -> TimeRange {
        let now = Local::now();
        let start_of_today = now
            .date_naive()
            .and_time(NaiveTime::MIN)
            .and_local_timezone(Local)
            .single()
            .unwrap_or(now);

        match self {
            Period::Today => TimeRange {
                since: start_of_today,
                until: None,
            },
            Period::Yesterday => {
                let yesterday_start = start_of_today - Duration::days(1);
                TimeRange {
                    since: yesterday_start,
                    until: Some(start_of_today),
                }
            }
            Period::Hours(h) => TimeRange {
                since: now - Duration::hours(i64::from(*h)),
                until: None,
            },
            Period::Days(d) => TimeRange {
                since: now - Duration::days(i64::from(*d)),
                until: None,
            },
            Period::Week => {
                let days_since_monday = now.weekday().num_days_from_monday() as i64;
                let monday = start_of_today - Duration::days(days_since_monday);
                TimeRange {
                    since: monday,
                    until: None,
                }
            }
        }
    }
}

fn end_of_day(date: NaiveDate) -> Result<DateTime<Local>, String> {
    let time = NaiveTime::from_hms_opt(23, 59, 59).unwrap_or(NaiveTime::MIN);
    date.and_time(time)
        .and_local_timezone(Local)
        .single()
        .ok_or_else(|| format!("Cannot convert {date} to local time"))
}

fn start_of_day(date: NaiveDate) -> Result<DateTime<Local>, String> {
    date.and_time(NaiveTime::MIN)
        .and_local_timezone(Local)
        .single()
        .ok_or_else(|| format!("Cannot convert {date} to local time"))
}

impl TimeRange {
    /// Build a range from two explicit dates (both inclusive).
    pub fn from_dates(since: NaiveDate, until: NaiveDate) -> Result<Self, String> {
        if since > until {
            return Err(format!(
                "--since ({since}) must be on or before --until ({until})"
            ));
        }
        Ok(TimeRange {
            since: start_of_day(since)?,
            until: Some(end_of_day(until)?),
        })
    }

    /// Build a range from a start date to now.
    pub fn from_since_date(since: NaiveDate) -> Result<Self, String> {
        Ok(TimeRange {
            since: start_of_day(since)?,
            until: None,
        })
    }

    /// Override the upper bound of an existing range.
    pub fn with_until_date(self, until: NaiveDate) -> Result<Self, String> {
        let until_dt = end_of_day(until)?;
        if self.since >= until_dt {
            return Err(format!("--since must be before --until ({until})"));
        }
        Ok(TimeRange {
            since: self.since,
            until: Some(until_dt),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Timelike, Weekday};

    #[test]
    fn parse_today() {
        assert!(matches!(Period::from_str("today"), Ok(Period::Today)));
    }

    #[test]
    fn parse_yesterday() {
        assert!(matches!(
            Period::from_str("yesterday"),
            Ok(Period::Yesterday)
        ));
    }

    #[test]
    fn parse_week() {
        assert!(matches!(Period::from_str("week"), Ok(Period::Week)));
    }

    #[test]
    fn parse_hours() {
        let period = Period::from_str("24h");
        assert!(matches!(period, Ok(Period::Hours(24))));
    }

    #[test]
    fn parse_days() {
        let period = Period::from_str("7d");
        assert!(matches!(period, Ok(Period::Days(7))));
    }

    #[test]
    fn parse_custom_days() {
        let period = Period::from_str("14d");
        assert!(matches!(period, Ok(Period::Days(14))));
    }

    #[test]
    fn parse_invalid_returns_error() {
        assert!(Period::from_str("invalid").is_err());
    }

    #[test]
    fn parse_invalid_number_returns_error() {
        assert!(Period::from_str("abch").is_err());
    }

    #[test]
    fn today_range_starts_at_midnight() {
        let range = Period::Today.to_time_range();
        assert_eq!(range.since.time().hour(), 0);
        assert_eq!(range.since.time().minute(), 0);
        assert!(range.until.is_none());
    }

    #[test]
    fn yesterday_range_has_both_bounds() {
        let range = Period::Yesterday.to_time_range();
        assert!(range.until.is_some());
        let until = range.until.as_ref().unwrap_or(&range.since);
        assert!(range.since < *until);
    }

    #[test]
    fn hours_range_is_in_past() {
        let range = Period::Hours(24).to_time_range();
        assert!(range.since < Local::now());
        assert!(range.until.is_none());
    }

    #[test]
    fn week_range_starts_on_monday() {
        let range = Period::Week.to_time_range();
        assert_eq!(range.since.weekday(), Weekday::Mon);
    }

    #[test]
    fn from_dates_valid_range() {
        let since = NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date");
        let until = NaiveDate::from_ymd_opt(2026, 3, 10).expect("valid date");
        let range = TimeRange::from_dates(since, until).expect("valid range");
        assert_eq!(range.since.date_naive(), since);
        assert_eq!(range.since.time().hour(), 0);
        let until_dt = range.until.expect("should have until");
        assert_eq!(until_dt.date_naive(), until);
        assert_eq!(until_dt.time().hour(), 23);
        assert_eq!(until_dt.time().minute(), 59);
    }

    #[test]
    fn from_dates_same_day_is_valid() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).expect("valid date");
        let range = TimeRange::from_dates(date, date).expect("same-day range");
        assert_eq!(range.since.date_naive(), date);
        let until_dt = range.until.expect("should have until");
        assert_eq!(until_dt.date_naive(), date);
        assert!(range.since < until_dt);
    }

    #[test]
    fn from_dates_inverted_errors() {
        let since = NaiveDate::from_ymd_opt(2026, 3, 10).expect("valid date");
        let until = NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date");
        let err = TimeRange::from_dates(since, until).unwrap_err();
        assert!(err.contains("must be on or before"));
    }

    #[test]
    fn from_since_date_open_ended() {
        let since = NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date");
        let range = TimeRange::from_since_date(since).expect("valid range");
        assert_eq!(range.since.date_naive(), since);
        assert!(range.until.is_none());
    }

    #[test]
    fn with_until_date_overrides_end() {
        let range = Period::Week.to_time_range();
        let until = NaiveDate::from_ymd_opt(2030, 12, 31).expect("valid date");
        let capped = range.with_until_date(until).expect("valid range");
        let until_dt = capped.until.expect("should have until");
        assert_eq!(until_dt.date_naive(), until);
    }

    #[test]
    fn with_until_date_before_since_errors() {
        let range = Period::Today.to_time_range();
        let until = NaiveDate::from_ymd_opt(2020, 1, 1).expect("valid date");
        assert!(range.with_until_date(until).is_err());
    }
}
