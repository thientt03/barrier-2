use chrono::{Local, NaiveDate, NaiveDateTime};

/// Mọi thời điểm lưu trong DB là giờ địa phương, định dạng này.
pub const FMT: &str = "%Y-%m-%d %H:%M:%S";

pub fn now() -> NaiveDateTime {
    Local::now().naive_local()
}

pub fn fmt(dt: NaiveDateTime) -> String {
    dt.format(FMT).to_string()
}

/// Nhận "YYYY-MM-DD HH:MM:SS" hoặc "YYYY-MM-DD" (ngày thuần = hết ngày đó).
pub fn parse(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, FMT) {
        return Some(dt);
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(23, 59, 59))
}

/// Đường dẫn ảnh tương đối: photos/YYYY/MM/DD/<entry|exit>_<session_id>.jpg
pub fn photo_rel(at: NaiveDateTime, kind: &str, session_id: &str) -> String {
    format!("photos/{}/{}_{}.jpg", at.format("%Y/%m/%d"), kind, session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_date_and_datetime() {
        assert_eq!(fmt(parse("2026-11-25").unwrap()), "2026-11-25 23:59:59");
        assert_eq!(fmt(parse("2026-11-25 08:15:32").unwrap()), "2026-11-25 08:15:32");
        assert!(parse("abc").is_none());
    }

    #[test]
    fn photo_path_layout() {
        let at = parse("2026-11-25 08:15:32").unwrap();
        assert_eq!(photo_rel(at, "entry", "abc"), "photos/2026/11/25/entry_abc.jpg");
    }
}
