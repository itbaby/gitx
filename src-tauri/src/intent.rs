use regex::Regex;
use std::sync::LazyLock;

// Only `parse_time_range` is still used — the natural-language intent
// classification (`parse_intent`, `Intent`, `IntentType`) was superseded by
// the agent's tool-calling loop and is no longer called from the frontend.
// Kept this module trimmed to the single remaining helper to avoid dragging
// dead regex machinery around.

static TIME_RANGE_PARSE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d+)([dhm])").expect("TIME_RANGE_PARSE regex"));

// ============================================================
// Time Range Parsing (used by file-history and tools)
// ============================================================

pub fn parse_time_range(time_range: &str) -> Result<i64, String> {
    if time_range.is_empty() {
        return Err("empty time range".to_string());
    }

    let caps = TIME_RANGE_PARSE
        .captures(time_range)
        .ok_or_else(|| format!("invalid time range format: {}", time_range))?;

    let value: i64 = caps
        .get(1)
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(3);
    let unit = caps.get(2).map(|m| m.as_str()).unwrap_or("d");

    let now = chrono::Local::now();
    let since = match unit {
        "d" => now - chrono::Duration::days(value),
        "h" => now - chrono::Duration::hours(value),
        "m" => now - chrono::Duration::minutes(value),
        _ => return Err(format!("invalid time unit: {}", unit)),
    };

    Ok(since.timestamp())
}

#[cfg(test)]
mod tests {
    use super::parse_time_range;

    /// Assert that `range` parses to a timestamp roughly `expected_secs` before
    /// now. Both sides call `chrono::Local::now()` independently (the helper
    /// under test reads now internally, the assertion reads it here), so we
    /// allow a few seconds of skew rather than checking an exact value. This
    /// catches magnitude regressions (e.g. `3d` resolving to 3 hours ago)
    /// that a bare `is_ok()` assertion would silently pass.
    fn assert_seconds_ago(range: &str, expected_secs: i64) {
        let parsed = parse_time_range(range).unwrap_or_else(|e| panic!("{}: {}", range, e));
        let now = chrono::Local::now().timestamp();
        let actual_diff = now - parsed;
        assert!(
            (actual_diff - expected_secs).abs() <= 5,
            "range {:?}: expected ~{}s ago, got {}s ago (now={}, parsed={})",
            range,
            expected_secs,
            actual_diff,
            now,
            parsed
        );
    }

    #[test]
    fn parse_days_matches_requested_duration() {
        assert_seconds_ago("3d", 3 * 86_400);
        assert_seconds_ago("14d", 14 * 86_400);
    }

    #[test]
    fn parse_hours_and_minutes_match_requested_duration() {
        assert_seconds_ago("24h", 24 * 3_600);
        assert_seconds_ago("90m", 90 * 60);
    }

    #[test]
    fn zero_duration_returns_now() {
        // "0d" is now-minus-zero -> within a couple seconds of now.
        let parsed = parse_time_range("0d").unwrap();
        let now = chrono::Local::now().timestamp();
        assert!((parsed - now).abs() <= 5, "0d should equal now within 5s");
    }

    #[test]
    fn empty_rejected() {
        assert!(parse_time_range("").is_err());
    }

    #[test]
    fn bad_format_rejected() {
        assert!(parse_time_range("soon").is_err());
        assert!(parse_time_range("3weeks").is_err());
    }
}
