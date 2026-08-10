use std::sync::OnceLock;

use anyhow::{Result, bail};
use time::{Duration, OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

static LOCAL_OFFSET: OnceLock<UtcOffset> = OnceLock::new();

/// Captures the system's UTC offset for the rest of the process.
///
/// Must be called from `main` *before* the async runtime starts: `time`
/// refuses to read the system timezone once the process is multi-threaded, so
/// calling this later silently falls back to UTC. Falls back to UTC if the
/// offset is unavailable (e.g. no timezone configured).
pub fn init_local_offset() {
    let _ = LOCAL_OFFSET.set(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));
}

/// Now, in the offset captured by [`init_local_offset`] (UTC if uninitialized).
fn now_local() -> OffsetDateTime {
    let offset = LOCAL_OFFSET.get().copied().unwrap_or(UtcOffset::UTC);
    OffsetDateTime::now_utc().to_offset(offset)
}

/// Parses a date string into an ISO 8601 formatted string for the Linear API.
///
/// Accepts:
/// - ISO 8601 dates: `2026-03-24`, `2026-03-24T10:00:00Z`
/// - Relative shorthand: `3d` (3 days ago), `1w` (1 week ago), `2h` (2 hours ago)
pub fn parse_date(input: &str) -> Result<String> {
    let input = input.trim();

    // Try relative shorthand first (e.g., 3d, 1w, 2h)
    if let Some(duration) = parse_relative(input) {
        let now = OffsetDateTime::now_utc();
        let target = now - duration;
        return Ok(target.format(&Rfc3339)?);
    }

    // Try ISO 8601 date-only format (YYYY-MM-DD)
    if input.len() == 10 && input.chars().nth(4) == Some('-') && input.chars().nth(7) == Some('-') {
        // Validate by parsing
        let parts: Vec<&str> = input.split('-').collect();
        if parts.len() == 3 {
            let year: i32 = parts[0]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid year"))?;
            let month: u8 = parts[1]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid month"))?;
            let day: u8 = parts[2]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid day"))?;

            if !(1..=12).contains(&month) {
                bail!("Invalid month: {}", month);
            }
            if !(1..=31).contains(&day) {
                bail!("Invalid day: {}", day);
            }

            // Return as ISO 8601 with midnight UTC
            return Ok(format!("{:04}-{:02}-{:02}T00:00:00Z", year, month, day));
        }
    }

    // Try full ISO 8601 datetime (already valid for Linear API)
    if input.contains('T') {
        // Validate by attempting to parse as RFC 3339
        match OffsetDateTime::parse(input, &Rfc3339) {
            Ok(dt) => return Ok(dt.format(&Rfc3339)?),
            Err(_) => {
                // Try adding Z if missing timezone
                let with_z =
                    if !input.ends_with('Z') && !input.contains('+') && !input.contains('-') {
                        format!("{}Z", input)
                    } else {
                        input.to_string()
                    };
                if let Ok(dt) = OffsetDateTime::parse(&with_z, &Rfc3339) {
                    return Ok(dt.format(&Rfc3339)?);
                }
            }
        }
    }

    bail!(
        "Invalid date format: '{}'. Expected ISO 8601 (e.g., 2026-03-24) or relative (e.g., 3d, 1w, 2h)",
        input
    )
}

/// Parses a date string and returns it as YYYY-MM-DD (TimelessDate).
///
/// Accepts the same inputs as [`parse_date`] but discards the time portion.
/// Use this for Linear API fields typed as `TimelessDate` (e.g., project
/// `startDate` / `targetDate`).
pub fn parse_date_only(input: &str) -> Result<String> {
    let full = parse_date(input)?;
    Ok(full[..10].to_string())
}

/// Parses a *deadline* date string and returns it as YYYY-MM-DD (TimelessDate).
///
/// Same inputs as [`parse_date_only`], with three differences, all because a
/// deadline names a calendar day the user is looking forward to rather than a
/// timestamp in the past:
/// - relative shorthand counts **forward**: `1w` is a week from today
/// - it counts from the *local* day (see [`init_local_offset`]), so `1d` is
///   tomorrow by the user's calendar, not by UTC's
/// - zero is valid: `0d` means today
///
/// Note `1m` is 30 days, not a calendar month — inherited from the shorthand
/// the list filters use. Durations that overflow the representable range are
/// an error, not a panic.
pub fn parse_future_date_only(input: &str) -> Result<String> {
    let input = input.trim();

    match parse_relative_forward(input)? {
        Some(duration) => {
            let target = now_local().checked_add(duration).ok_or_else(|| {
                anyhow::anyhow!("Due date '{}' is too far in the future to represent", input)
            })?;
            Ok(target.format(&Rfc3339)?[..10].to_string())
        }
        // Not relative shorthand at all — try the absolute formats.
        None => parse_date_only(input),
    }
}

/// Relative shorthand for a deadline: like [`parse_relative`], but accepts a
/// zero count (`0d` = today) and reports overflow as an error rather than
/// panicking the way `Duration::weeks` does.
///
/// `Ok(None)` means "not relative shorthand, try another format"; `Err` means
/// "recognizably shorthand, but out of range" — the caller must not retry
/// those through [`parse_date_only`], which would hit the panic.
fn parse_relative_forward(input: &str) -> Result<Option<Duration>> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(None);
    }

    let (num_str, unit) = input.split_at(input.len() - 1);
    if !matches!(unit, "h" | "d" | "w" | "m") {
        return Ok(None);
    }
    let Ok(num) = num_str.parse::<i64>() else {
        return Ok(None);
    };

    if num < 0 {
        bail!(
            "Due date '{}' is negative; a due date must not be in the past",
            input
        );
    }

    let seconds_per_unit = match unit {
        "h" => 3_600,
        "d" => 86_400,
        "w" => 604_800,
        "m" => 30 * 86_400, // Approximate month, matching the shorthand elsewhere
        _ => unreachable!("unit checked above"),
    };

    let seconds = num
        .checked_mul(seconds_per_unit)
        .ok_or_else(|| anyhow::anyhow!("Due date '{}' is too far in the future", input))?;

    Ok(Some(Duration::seconds(seconds)))
}

/// Adds a relative duration (e.g., "1w", "10d") to an ISO 8601 date string.
pub fn add_duration_to_date(date_str: &str, duration_str: &str) -> Result<String> {
    let duration = parse_relative(duration_str).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid duration: '{}'. Expected format like 1w, 2w, 10d",
            duration_str
        )
    })?;
    let start = parse_date(date_str)?;
    let start_dt = OffsetDateTime::parse(&start, &Rfc3339)?;
    let end = start_dt + duration;
    Ok(end.format(&Rfc3339)?)
}

/// Parses relative time shorthand like "3d", "1w", "2h" into a Duration.
fn parse_relative(input: &str) -> Option<Duration> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let (num_str, unit) = input.split_at(input.len() - 1);
    let num: i64 = num_str.parse().ok()?;

    if num <= 0 {
        return None;
    }

    match unit {
        "h" => Some(Duration::hours(num)),
        "d" => Some(Duration::days(num)),
        "w" => Some(Duration::weeks(num)),
        "m" => Some(Duration::days(num * 30)), // Approximate month
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_iso_date() {
        let result = parse_date("2026-03-24").unwrap();
        assert_eq!(result, "2026-03-24T00:00:00Z");
    }

    #[test]
    fn parse_iso_datetime() {
        let result = parse_date("2026-03-24T10:30:00Z").unwrap();
        assert!(result.contains("2026-03-24"));
        assert!(result.contains("10:30:00"));
    }

    #[test]
    fn parse_relative_days() {
        let result = parse_date("3d").unwrap();
        // Should be a valid RFC3339 timestamp
        assert!(result.contains("T"));
        assert!(result.ends_with("Z"));
    }

    #[test]
    fn parse_relative_weeks() {
        let result = parse_date("1w").unwrap();
        assert!(result.contains("T"));
        assert!(result.ends_with("Z"));
    }

    #[test]
    fn parse_relative_hours() {
        let result = parse_date("2h").unwrap();
        assert!(result.contains("T"));
        assert!(result.ends_with("Z"));
    }

    #[test]
    fn invalid_date_errors() {
        assert!(parse_date("invalid").is_err());
        assert!(parse_date("2026-13-01").is_err()); // Invalid month
        assert!(parse_date("2026-01-32").is_err()); // Invalid day
    }

    #[test]
    fn parse_relative_months() {
        let result = parse_date("2m").unwrap();
        assert!(result.contains("T"));
        assert!(result.ends_with("Z"));
    }

    #[test]
    fn zero_relative_fails() {
        assert!(parse_date("0d").is_err());
    }

    #[test]
    fn negative_relative_fails() {
        assert!(parse_date("-3d").is_err());
    }

    #[test]
    fn add_duration_one_week() {
        let result = add_duration_to_date("2026-04-07", "1w").unwrap();
        assert!(result.starts_with("2026-04-14"));
    }

    #[test]
    fn add_duration_ten_days() {
        let result = add_duration_to_date("2026-04-01", "10d").unwrap();
        assert!(result.starts_with("2026-04-11"));
    }

    #[test]
    fn add_duration_invalid() {
        assert!(add_duration_to_date("2026-04-01", "abc").is_err());
    }

    #[test]
    fn parse_date_only_iso() {
        assert_eq!(parse_date_only("2026-03-24").unwrap(), "2026-03-24");
    }

    #[test]
    fn parse_date_only_iso_datetime() {
        assert_eq!(
            parse_date_only("2026-03-24T10:30:00Z").unwrap(),
            "2026-03-24"
        );
    }

    #[test]
    fn parse_date_only_relative() {
        let result = parse_date_only("3d").unwrap();
        assert_eq!(result.len(), 10);
        assert_eq!(result.chars().nth(4), Some('-'));
        assert_eq!(result.chars().nth(7), Some('-'));
    }

    #[test]
    fn parse_date_only_invalid() {
        assert!(parse_date_only("not-a-date").is_err());
    }

    #[test]
    fn future_date_only_iso_is_verbatim() {
        assert_eq!(parse_future_date_only("2026-09-30").unwrap(), "2026-09-30");
    }

    #[test]
    fn future_date_only_relative_counts_forward() {
        let today = OffsetDateTime::now_utc().format(&Rfc3339).unwrap()[..10].to_string();
        let future = parse_future_date_only("1w").unwrap();
        let past = parse_date_only("1w").unwrap();
        assert!(future > today, "{} should be after {}", future, today);
        assert!(past < today, "{} should be before {}", past, today);
    }

    #[test]
    fn future_date_only_invalid() {
        assert!(parse_future_date_only("not-a-date").is_err());
    }

    #[test]
    fn future_date_only_zero_is_today() {
        let today = now_local().format(&Rfc3339).unwrap()[..10].to_string();
        assert_eq!(parse_future_date_only("0d").unwrap(), today);
        assert_eq!(parse_future_date_only("0w").unwrap(), today);
    }

    #[test]
    fn future_date_only_rejects_negative() {
        assert!(parse_future_date_only("-1w").is_err());
    }

    #[test]
    fn future_date_only_overflow_errors_not_panics() {
        // `parse_date` panics on these; the deadline path must not.
        assert!(parse_future_date_only("9999999999w").is_err());
        assert!(parse_future_date_only("100000000000000w").is_err());
    }

    #[test]
    fn future_date_only_counts_from_local_day() {
        // Whatever offset was captured, the anchor is that day — not UTC's,
        // unless they happen to coincide.
        let local_today = now_local().format(&Rfc3339).unwrap()[..10].to_string();
        assert_eq!(parse_future_date_only("0d").unwrap(), local_today);
    }
}
