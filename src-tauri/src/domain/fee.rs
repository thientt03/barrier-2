use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct Rule {
    pub minute_from: i64,
    pub minute_to: i64,
    pub price_per_unit: i64,
    pub unit_minutes: i64,
}

/// Công thức theo spec 7.3: mỗi khung giá tính phần thời gian giao với khung đó,
/// làm tròn lên theo `unit_minutes`, rồi cộng các khung lại.
pub fn calc_fee(rules: &[Rule], duration_minutes: i64) -> i64 {
    rules
        .iter()
        .map(|r| {
            let overlap = duration_minutes.min(r.minute_to) - r.minute_from;
            if overlap > 0 {
                let unit = r.unit_minutes.max(1);
                ((overlap + unit - 1) / unit) * r.price_per_unit
            } else {
                0
            }
        })
        .sum()
}

/// Vé VIP và vé tháng còn hạn (hoặc không giới hạn) miễn phí.
pub fn is_free(card_type: &str, valid_to: Option<NaiveDateTime>, at: NaiveDateTime) -> bool {
    match card_type {
        "vip" => true,
        "monthly" => valid_to.map_or(true, |v| v >= at),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::parse;

    fn r(from: i64, to: i64, price: i64, unit: i64) -> Rule {
        Rule { minute_from: from, minute_to: to, price_per_unit: price, unit_minutes: unit }
    }

    fn motorbike() -> Vec<Rule> {
        vec![r(0, 120, 5000, 120), r(120, 1440, 3000, 60), r(1440, 999_999, 50_000, 1440)]
    }

    #[test]
    fn spec_example_three_hours_motorbike_is_8000() {
        assert_eq!(calc_fee(&motorbike(), 180), 8000);
    }

    #[test]
    fn one_minute_still_charges_one_unit() {
        assert_eq!(calc_fee(&motorbike(), 1), 5000);
    }

    #[test]
    fn partial_hour_rounds_up() {
        // 2h01 -> 5000 + 1 giờ tiếp theo
        assert_eq!(calc_fee(&motorbike(), 121), 8000);
    }

    #[test]
    fn zero_duration_is_free() {
        assert_eq!(calc_fee(&motorbike(), 0), 0);
    }

    #[test]
    fn zero_unit_minutes_does_not_panic() {
        assert_eq!(calc_fee(&[r(0, 60, 1000, 0)], 30), 30_000);
    }

    #[test]
    fn free_rules() {
        let now = parse("2026-11-25 10:00:00").unwrap();
        assert!(is_free("vip", None, now));
        assert!(is_free("monthly", None, now));
        assert!(is_free("monthly", parse("2026-12-01"), now));
        assert!(!is_free("monthly", parse("2026-11-01"), now));
        assert!(!is_free("guest", None, now));
    }
}
