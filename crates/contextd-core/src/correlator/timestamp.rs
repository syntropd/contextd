//! Strict timestamp parsers for DNF and DPKG package manager log lines.
//!
//! Returns the timestamp expressed as microseconds since the UNIX epoch.
//! Returns `None` on any malformed input; callers should treat that as
//! "skip this line" rather than "log time" or "now".

const SEC_PER_MIN: u64 = 60;
const SEC_PER_HOUR: u64 = 3600;
const SEC_PER_DAY: u64 = 86_400;

const DAYS_BEFORE_MONTH: [u64; 13] =
    [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(month: u64, year: u64) -> u64 {
    let mut dim = DAYS_BEFORE_MONTH[month as usize] - DAYS_BEFORE_MONTH[(month - 1) as usize];
    if month == 2 && is_leap(year) {
        dim += 1;
    }
    dim
}

fn days_from_epoch(year: u64, month: u64, day: u64) -> Option<u64> {
    if !(1..=12).contains(&month) || year < 1970 {
        return None;
    }
    if day < 1 || day > days_in_month(month, year) {
        return None;
    }
    let mut doy = DAYS_BEFORE_MONTH[(month - 1) as usize];
    if month > 2 && is_leap(year) {
        doy += 1;
    }
    doy += day - 1;
    let mut days = 0u64;
    for yr in 1970..year {
        days += if is_leap(yr) { 366 } else { 365 };
    }
    Some(days + doy)
}

fn parse_uint(b: &[u8]) -> Option<u64> {
    if b.is_empty() {
        return None;
    }
    let mut v: u64 = 0;
    for &c in b {
        if !c.is_ascii_digit() {
            return None;
        }
        v = v.checked_mul(10)?.checked_add((c - b'0') as u64)?;
    }
    Some(v)
}

/// Parses ISO 8601 UTC timestamp. Accepts either:
/// - `YYYY-MM-DDTHH:MM:SS` (19 bytes, naive UTC)
/// - `YYYY-MM-DDTHH:MM:SSZ` (20 bytes, explicit UTC)
/// - `YYYY-MM-DDTHH:MM:SS+HHMM` or `-HHMM` (24 bytes, numeric offset)
/// Trailing junk beyond the timezone marker is rejected. Non-UTC
/// offsets are converted to UTC: `+0000` is a no-op, `+0530` adds
/// 5h30m to the resulting timestamp.
pub fn parse_iso8601_utc(s: &str) -> Option<u64> {
    let b = s.as_bytes();
    if b.len() == 19 {
        parse_iso8601_19(&b[0..19])
    } else if b.len() == 20 && b[19] == b'Z' {
        parse_iso8601_19(&b[0..19])
    } else if b.len() == 24 && (b[19] == b'+' || b[19] == b'-') {
        let naive = parse_iso8601_19(&b[0..19])?;
        let sign: i64 = if b[19] == b'+' { 1 } else { -1 };
        let hh = parse_uint(&b[20..22])?;
        let mm = parse_uint(&b[22..24])?;
        let offset_secs = sign * (hh as i64 * 3600 + mm as i64 * 60);
        // The naive timestamp is interpreted as local time of that
        // offset; to convert to UTC, subtract the offset.
        let naive_secs = naive / 1_000_000;
        // `naive_secs` is u64; clamp offset to i64's negative range via
        // saturating arithmetic so a wildly negative offset cannot panic.
        let utc_secs = if offset_secs < 0 {
            naive_secs.saturating_sub(offset_secs.unsigned_abs())
        } else {
            naive_secs.saturating_sub(offset_secs as u64)
        };
        Some(utc_secs.saturating_mul(1_000_000))
    } else {
        None
    }
}

fn parse_iso8601_19(b: &[u8]) -> Option<u64> {
    if b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    if b[10] != b'T' && b[10] != b' ' {
        return None;
    }
    if b[13] != b':' || b[16] != b':' {
        return None;
    }
    let year = parse_uint(&b[0..4])?;
    let month = parse_uint(&b[5..7])?;
    let day = parse_uint(&b[8..10])?;
    let hour = parse_uint(&b[11..13])?;
    let minute = parse_uint(&b[14..16])?;
    let second = parse_uint(&b[17..19])?;
    if hour >= 24 || minute >= 60 || second >= 60 {
        return None;
    }
    let day_offset = days_from_epoch(year, month, day)?;
    let secs = day_offset * SEC_PER_DAY + hour * SEC_PER_HOUR + minute * SEC_PER_MIN + second;
    Some(secs * 1_000_000)
}

/// Parses DPKG `YYYY-MM-DD` (10 bytes) and `HH:MM:SS` (8 bytes).
/// Both arguments must be exactly the right length.
pub fn parse_dpkg_date_time(date: &str, time: &str) -> Option<u64> {
    let d = date.as_bytes();
    let t = time.as_bytes();
    if d.len() != 10 || t.len() != 8 || d[4] != b'-' || d[7] != b'-' || t[2] != b':' || t[5] != b':' {
        return None;
    }
    let year = parse_uint(&d[0..4])?;
    let month = parse_uint(&d[5..7])?;
    let day = parse_uint(&d[8..10])?;
    let hour = parse_uint(&t[0..2])?;
    let minute = parse_uint(&t[3..5])?;
    let second = parse_uint(&t[6..8])?;
    if hour >= 24 || minute >= 60 || second >= 60 {
        return None;
    }
    let day_offset = days_from_epoch(year, month, day)?;
    let secs = day_offset * SEC_PER_DAY + hour * SEC_PER_HOUR + minute * SEC_PER_MIN + second;
    Some(secs * 1_000_000)
}
