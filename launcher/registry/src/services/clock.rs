use std::time::{SystemTime, UNIX_EPOCH};

/// The current time as RFC 3339 UTC text.
pub struct Clock;

impl Clock {
    pub fn now() -> String {
        let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| elapsed.as_secs()).unwrap_or(0);
        Self::format(seconds)
    }

    pub fn format(unix_seconds: u64) -> String {
        let days = (unix_seconds / 86_400) as i64;
        let rest = unix_seconds % 86_400;
        let (year, month, day) = Self::civil(days);
        format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z", rest / 3600, rest % 3600 / 60, rest % 60)
    }

    // days since 1970-01-01 to a proleptic Gregorian date
    fn civil(days: i64) -> (i64, i64, i64) {
        let shifted = days + 719_468;
        let era = shifted.div_euclid(146_097);
        let day_of_era = shifted.rem_euclid(146_097);
        let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_part = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_part + 2) / 5 + 1;
        let month = if month_part < 10 { month_part + 3 } else { month_part - 9 };
        let year = year_of_era + era * 400 + i64::from(month <= 2);
        (year, month, day)
    }
}
