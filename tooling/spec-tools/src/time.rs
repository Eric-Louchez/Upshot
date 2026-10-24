//! UTC calendar arithmetic for date-based ids: no time zones, no dependencies.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// 2026-01-01T00:00Z, the epoch of date-based ids, in seconds since the Unix epoch.
pub const EPOCH_UNIX: i64 = 1_767_225_600;

/// A calendar date (proleptic Gregorian).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

impl Date {
    /// Exactly `yyyy-mm-dd` with ASCII digits, and a day that exists; `None` for anything else.
    #[must_use]
    pub fn parse_iso(text: &str) -> Option<Self> {
        let b = text.as_bytes();
        let shape = b.len() == 10
            && b[4] == b'-'
            && b[7] == b'-'
            && b.iter()
                .enumerate()
                .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
        if !shape {
            return None;
        }
        let year = text[0..4].parse().ok()?;
        let month = text[5..7].parse().ok()?;
        let day = text[8..10].parse().ok()?;
        ((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month))
            .then_some(Self { year, month, day })
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A UTC time to the minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime {
    pub date: Date,
    pub hour: u32,
    pub minute: u32,
}

impl fmt::Display for DateTime {
    /// `yyyy-mm-dd HH:mm`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:02}:{:02}", self.date, self.hour, self.minute)
    }
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 of a civil date (H. Hinnant's algorithm).
#[must_use]
pub fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(month);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The civil date of a day count since 1970-01-01 (H. Hinnant's algorithm).
#[must_use]
pub fn civil_from_days(days: i64) -> Date {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    Date {
        year,
        month: u32::try_from(month).unwrap_or(1),
        day: u32::try_from(day).unwrap_or(1),
    }
}

/// Whole minutes since the epoch (UTC) at `unix_seconds`; `None` before 2026-01-01T00:01Z (no date-based id).
#[must_use]
pub fn minutes(unix_seconds: i64) -> Option<u64> {
    let m = (unix_seconds - EPOCH_UNIX).div_euclid(60);
    u64::try_from(m).ok().filter(|&m| m >= 1)
}

/// The UTC time a date-based id stands for.
#[must_use]
pub fn time_of(id: u64) -> DateTime {
    let id = i64::try_from(id).unwrap_or(i64::MAX / 60);
    let total = EPOCH_UNIX / 60 + id;
    let days = total.div_euclid(1440);
    let in_day = total.rem_euclid(1440);
    DateTime {
        date: civil_from_days(days),
        hour: u32::try_from(in_day / 60).unwrap_or(0),
        minute: u32::try_from(in_day % 60).unwrap_or(0),
    }
}

/// The system clock, in seconds since the Unix epoch.
#[must_use]
pub fn now_unix() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_secs()).unwrap_or(i64::MAX),
        Err(e) => -i64::try_from(e.duration().as_secs()).unwrap_or(i64::MAX),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unix(y: i64, mo: u32, d: u32, h: i64, mi: i64, s: i64) -> i64 {
        days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + s
    }

    #[test]
    fn the_epoch_is_2026_01_01() {
        assert_eq!(unix(2026, 1, 1, 0, 0, 0), EPOCH_UNIX);
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(
            civil_from_days(days_from_civil(2024, 2, 29)),
            Date {
                year: 2024,
                month: 2,
                day: 29
            }
        );
    }

    #[test]
    fn dates_parse_exactly() {
        assert_eq!(
            Date::parse_iso("2026-10-06").map(|d| d.to_string()),
            Some("2026-10-06".into())
        );
        for bad in [
            "2026-02-30",
            "2026-13-01",
            "2026-00-10",
            "2026-1-01",
            "26-10-06",
            "2026/10/06",
            "２026-10-06",
        ] {
            assert_eq!(Date::parse_iso(bad), None, "{bad}");
        }
        assert!(Date::parse_iso("2024-02-29").is_some());
    }

    #[test]
    fn minutes_and_back() {
        let at = unix(2026, 10, 18, 7, 30, 0);
        let m = 290 * 1440 + 7 * 60 + 30;
        assert_eq!(minutes(at), Some(m));
        assert_eq!(minutes(at + 59), Some(m));
        assert_eq!(time_of(m).to_string(), "2026-10-18 07:30");
        assert_eq!(minutes(EPOCH_UNIX), None);
        assert_eq!(minutes(EPOCH_UNIX - 3600), None);
        assert_eq!(minutes(EPOCH_UNIX + 60), Some(1));
    }
}
