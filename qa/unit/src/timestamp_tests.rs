//! Unit QA tests for the strict timestamp parsers used by the
//! correlator to assign real wall-clock times to package events.

#[cfg(test)]
mod tests {
    use contextd_core::correlator::timestamp::{parse_dpkg_date_time, parse_iso8601_utc};

    #[test]
    fn test_parse_iso8601_valid_utc() {
        // 2024-01-01T00:00:00Z == 1_704_067_200 s
        let ts = parse_iso8601_utc("2024-01-01T00:00:00Z");
        assert_eq!(ts, Some(1_704_067_200 * 1_000_000));
    }

    #[test]
    fn test_parse_iso8601_space_separator() {
        let ts = parse_iso8601_utc("2024-09-25 03:54:25Z");
        assert!(ts.is_some());
    }

    #[test]
    fn test_parse_iso8601_invalid_inputs() {
        assert!(parse_iso8601_utc("not a date").is_none());
        assert!(parse_iso8601_utc("2024-13-01T00:00:00Z").is_none());
        assert!(parse_iso8601_utc("2024-01-32T00:00:00Z").is_none());
        assert!(parse_iso8601_utc("2024-01-01T25:00:00Z").is_none());
        assert!(parse_iso8601_utc("").is_none());
        // Invalid calendar dates that the days-in-month check must reject.
        assert!(parse_iso8601_utc("2024-02-30T00:00:00Z").is_none(), "Feb 30 should be rejected");
        assert!(parse_iso8601_utc("2023-02-29T00:00:00Z").is_none(), "Feb 29 in non-leap year");
        assert!(parse_iso8601_utc("2024-04-31T00:00:00Z").is_none(), "Apr 31 should be rejected");
        // Trailing junk beyond the optional Z must be rejected.
        assert!(parse_iso8601_utc("2024-01-01T00:00:00EXTRA").is_none(), "trailing junk rejected");
        assert!(parse_iso8601_utc("2024-01-01T00:00:00 ").is_none(), "trailing space rejected");
    }

    #[test]
    fn test_parse_iso8601_accepts_naive_19_byte_form() {
        // The 19-byte naive form is accepted for compatibility with timestamps
        // that lack an explicit Z (e.g. Pacman log lines).
        let naive = parse_iso8601_utc("2024-01-01T00:00:00");
        let explicit = parse_iso8601_utc("2024-01-01T00:00:00Z");
        assert_eq!(naive, explicit);
    }

    #[test]
    fn test_parse_iso8601_accepts_numeric_offset() {
        // Pacman and other tools emit `+0000` instead of `Z`. The parser
        // must accept it and treat it as UTC.
        let z = parse_iso8601_utc("2024-01-01T00:00:00Z").unwrap();
        let plus = parse_iso8601_utc("2024-01-01T00:00:00+0000").unwrap();
        assert_eq!(z, plus);
    }

    #[test]
    fn test_parse_dpkg_date_time_valid() {
        let ts = parse_dpkg_date_time("2024-01-01", "00:00:00");
        assert_eq!(ts, Some(1_704_067_200 * 1_000_000));
    }

    #[test]
    fn test_parse_dpkg_date_time_invalid() {
        assert!(parse_dpkg_date_time("not-a-date", "00:00:00").is_none());
        assert!(parse_dpkg_date_time("2024-01-01", "bad-time").is_none());
        assert!(parse_dpkg_date_time("2024-01-01", "25:00:00").is_none());
    }
}