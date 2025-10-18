use chrono::{DateTime, FixedOffset, NaiveDate, Offset, Utc};

use crate::ParseError;

pub(crate) fn parse_date(date_str: &str) -> Result<DateTime<FixedOffset>, ParseError> {
    let formats = vec![
        "%d.%m.%Y",          // e.g., "26.10.2023"
        "%Y-%m-%d",          // e.g., "2023-10-26"
        "%Y-%m-%dT%H:%M:%S", // e.g., "2023-10-26T12:00:00"
    ];

    if let Ok(date) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(date);
    }
    for format in formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            // Construct datetime at midnight UTC+0 (you can change offset)
            let ndt = date
                .and_hms_opt(0, 0, 0)
                .ok_or(ParseError::InvalidFormat("Invalid date".into()))?;
            return Ok(DateTime::<FixedOffset>::from_naive_utc_and_offset(
                ndt,
                Utc.fix(),
            ));
        }
    }

    Err(ParseError::InvalidFormat("Invalid date".into()))
}
