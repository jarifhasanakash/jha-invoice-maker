//! Date parsing/formatting helpers (DD-MMM-YYYY display, YYYY-MM-DD storage).

use chrono::{Datelike, Local, NaiveDate};

pub const MONTH_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
pub const MONTH_LONG: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

fn month_from_name(s: &str) -> Option<u32> {
    let s = s.to_lowercase();
    let m = match s.as_str() {
        "jan" | "january" => 1,
        "feb" | "february" => 2,
        "mar" | "march" => 3,
        "apr" | "april" => 4,
        "may" => 5,
        "jun" | "june" => 6,
        "jul" | "july" => 7,
        "aug" | "august" => 8,
        "sep" | "sept" | "september" => 9,
        "oct" | "october" => 10,
        "nov" | "november" => 11,
        "dec" | "december" => 12,
        other => other.parse::<u32>().ok().filter(|m| (1..=12).contains(m))?,
    };
    Some(m)
}

/// Parses `YYYY-MM-DD`, `DD-MMM-YYYY`, ISO timestamps and a few loose formats.
pub fn parse(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // Strip a time component from ISO timestamps.
    let date_part = s.split(['T', ' ']).next().unwrap_or(s);
    let parts: Vec<&str> = date_part.split(['-', '/']).collect();
    if parts.len() == 3 {
        if parts[0].len() == 4 {
            let y = parts[0].parse().ok()?;
            let m = parts[1].parse().ok()?;
            let d = parts[2].parse().ok()?;
            return NaiveDate::from_ymd_opt(y, m, d);
        }
        if parts[2].len() == 4 {
            let d = parts[0].parse().ok()?;
            let m = month_from_name(parts[1])?;
            let y = parts[2].parse().ok()?;
            return NaiveDate::from_ymd_opt(y, m, d);
        }
    }
    for fmt in ["%d %b %Y", "%b %d %Y", "%B %d, %Y", "%d %B %Y"] {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
    }
    None
}

/// `16-Sep-2026`
pub fn display(d: NaiveDate) -> String {
    format!("{:02}-{}-{}", d.day(), MONTH_SHORT[d.month0() as usize], d.year())
}

/// `2026-09-16`
pub fn iso(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// Formats any stored date string for display; returns the input unchanged if unparseable.
pub fn display_str(s: &str) -> String {
    match parse(s) {
        Some(d) => display(d),
        None => s.to_string(),
    }
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    NaiveDate::from_ymd_opt(ny, nm, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(30)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_formats() {
        let d = NaiveDate::from_ymd_opt(2026, 9, 16).unwrap();
        assert_eq!(parse("2026-09-16"), Some(d));
        assert_eq!(parse("16-Sep-2026"), Some(d));
        assert_eq!(parse("16-september-2026"), Some(d));
        assert_eq!(parse("2026-09-16T10:00:00.000Z"), Some(d));
        assert_eq!(display(d), "16-Sep-2026");
        assert_eq!(iso(d), "2026-09-16");
        assert_eq!(days_in_month(2028, 2), 29);
    }
}
