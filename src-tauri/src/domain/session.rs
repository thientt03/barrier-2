use crate::domain::{audit, fee};
use crate::db::repo::{self, ActiveSession, Card};
use crate::error::{AppError, Result};
use crate::util;
use chrono::NaiveDateTime;
use rusqlite::{params, Connection};

/// Kiểm tra thẻ có được phép vào bãi (spec 7.1 bước 4). Trả về thẻ nếu hợp lệ.
pub fn validate_entry(conn: &Connection, code: &str, now: NaiveDateTime) -> Result<Card> {
    let card = repo::find_card(conn, code)?.ok_or_else(|| AppError::msg("Thẻ không hợp lệ"))?;
    if card.status != "active" {
        return Err(AppError::msg(format!("Thẻ đang ở trạng thái: {}", status_vi(&card.status))));
    }
    if card.card_type == "monthly" {
        if let Some(valid_to) = card.valid_to.as_deref().and_then(util::parse) {
            if valid_to < now {
                return Err(AppError::msg("Vé tháng đã hết hạn"));
            }
        }
    }
    if repo::find_active_session(conn, &card.id)?.is_some() {
        return Err(AppError::msg("Thẻ đang trong bãi (chưa quẹt ra)"));
    }
    Ok(card)
}

fn status_vi(s: &str) -> &str {
    match s {
        "lost" => "đã báo mất",
        "expired" => "hết hạn",
        "blocked" => "bị khóa",
        other => other,
    }
}

/// Tạo phiên đỗ xe + audit trong một transaction. Kiểm tra lại thẻ ngay trong transaction.
pub fn create_session(
    conn: &mut Connection,
    session_id: &str,
    card_id: &str,
    lane: &str,
    entry_time: NaiveDateTime,
    photo_rel: &str,
    operator_id: &str,
) -> Result<()> {
    let tx = conn.transaction()?;
    let card = validate_entry(&tx, card_id, entry_time)?;
    tx.execute(
        "INSERT INTO sessions (id, card_id, lane, entry_time, entry_photo_path, entry_operator, status) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'active')",
        params![session_id, card.id, lane, util::fmt(entry_time), photo_rel, operator_id],
    )?;
    audit::log(
        &tx,
        Some(session_id),
        Some(operator_id),
        "entry",
        Some(serde_json::json!({ "card_id": card.id, "lane": lane })),
    )?;
    tx.commit()?;
    Ok(())
}

/// Tìm thẻ và phiên đang đỗ của nó (spec 7.2 bước 3).
pub fn find_active_for_exit(conn: &Connection, code: &str) -> Result<(Card, ActiveSession)> {
    let card = repo::find_card(conn, code)?.ok_or_else(|| AppError::msg("Thẻ không hợp lệ"))?;
    let session = repo::find_active_session(conn, &card.id)?.ok_or_else(|| {
        AppError::msg("Không tìm thấy phiên đỗ xe của thẻ này (xe chưa vào bãi, hoặc cần xử lý mất thẻ)")
    })?;
    Ok((card, session))
}

fn vehicle_type_for(card: &Card, session: &ActiveSession) -> String {
    card.vehicle_type.clone().unwrap_or_else(|| {
        if session.lane.contains("bike") { "motorbike" } else { "car" }.to_string()
    })
}

/// Thời gian đỗ (phút, làm tròn lên, tối thiểu 1) và phí phải thu.
pub fn compute_exit(
    conn: &Connection,
    card: &Card,
    session: &ActiveSession,
    exit_time: NaiveDateTime,
) -> Result<(i64, i64)> {
    let secs = (exit_time - session.entry_time).num_seconds().max(0);
    let duration = ((secs + 59) / 60).max(1);

    let valid_to = card.valid_to.as_deref().and_then(util::parse);
    if fee::is_free(&card.card_type, valid_to, exit_time) {
        return Ok((duration, 0));
    }
    let vehicle = vehicle_type_for(card, session);
    let rules = repo::active_rules(conn, &vehicle)?;
    if rules.is_empty() {
        return Err(AppError::msg(format!("Chưa cấu hình bảng giá cho loại xe: {vehicle}")));
    }
    Ok((duration, fee::calc_fee(&rules, duration)))
}

/// Đóng phiên đỗ xe + audit trong một transaction.
pub fn complete_session(
    conn: &mut Connection,
    session_id: &str,
    exit_time: NaiveDateTime,
    exit_photo_rel: &str,
    operator_id: &str,
    duration_minutes: i64,
    fee: i64,
) -> Result<()> {
    let tx = conn.transaction()?;
    let payment = if fee == 0 { "free" } else { "cash" };
    let changed = tx.execute(
        "UPDATE sessions SET exit_time=?1, exit_photo_path=?2, exit_operator=?3, \
         duration_minutes=?4, fee=?5, fee_collected=?5, payment_method=?6, status='completed' \
         WHERE id=?7 AND status='active'",
        params![util::fmt(exit_time), exit_photo_rel, operator_id, duration_minutes, fee, payment, session_id],
    )?;
    if changed != 1 {
        return Err(AppError::msg("Phiên đỗ xe đã được đóng hoặc không tồn tại"));
    }
    audit::log(
        &tx,
        Some(session_id),
        Some(operator_id),
        "exit",
        Some(serde_json::json!({ "fee": fee, "duration_minutes": duration_minutes })),
    )?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth;
    use crate::db::open_memory;

    fn at(s: &str) -> NaiveDateTime {
        util::parse(s).unwrap()
    }

    fn setup() -> (Connection, String) {
        let mut conn = open_memory();
        let op = auth::create_first_manager(&mut conn, "admin", "matkhau123", "Admin").unwrap();
        for (id, ty, code, vt, valid_to) in [
            ("AAAA0001", "guest", "00001", Some("motorbike"), None),
            ("AAAA0002", "monthly", "00002", Some("car"), Some("2026-12-31")),
            ("AAAA0003", "monthly", "00003", Some("car"), Some("2026-01-01")),
            ("AAAA0004", "vip", "00004", Some("car"), None),
        ] {
            conn.execute(
                "INSERT INTO cards (id, card_type, display_code, vehicle_type, valid_to) VALUES (?1,?2,?3,?4,?5)",
                params![id, ty, code, vt, valid_to],
            )
            .unwrap();
        }
        (conn, op.id)
    }

    #[test]
    fn guest_entry_then_exit_charges_per_spec() {
        let (mut conn, op) = setup();
        let t_in = at("2026-11-25 08:00:00");
        validate_entry(&conn, "aaaa0001", t_in).unwrap();
        create_session(&mut conn, "s1", "AAAA0001", "entry_bike", t_in, "photos/x.jpg", &op).unwrap();

        // Thẻ đang trong bãi -> không cho vào lần 2
        assert!(validate_entry(&conn, "AAAA0001", t_in).is_err());

        let (card, session) = find_active_for_exit(&conn, "00001").unwrap(); // nhập mã in trên thẻ
        let t_out = at("2026-11-25 11:00:00");
        let (dur, fee) = compute_exit(&conn, &card, &session, t_out).unwrap();
        assert_eq!((dur, fee), (180, 8000));

        complete_session(&mut conn, &session.id, t_out, "photos/y.jpg", &op, dur, fee).unwrap();
        assert!(find_active_for_exit(&conn, "AAAA0001").is_err());
        // đóng lần 2 phải thất bại
        assert!(complete_session(&mut conn, "s1", t_out, "p", &op, dur, fee).is_err());
        // và thẻ vào lại được
        validate_entry(&conn, "AAAA0001", t_out).unwrap();

        let audits: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_logs WHERE action IN ('entry','exit')", [], |r| r.get(0))
            .unwrap();
        assert_eq!(audits, 2);
    }

    #[test]
    fn monthly_and_vip_are_free_and_expired_monthly_is_rejected() {
        let (mut conn, op) = setup();
        let now = at("2026-11-25 08:00:00");
        assert!(validate_entry(&conn, "AAAA0003", now).is_err()); // hết hạn

        create_session(&mut conn, "m1", "AAAA0002", "entry_car", now, "p", &op).unwrap();
        let (card, session) = find_active_for_exit(&conn, "AAAA0002").unwrap();
        assert_eq!(compute_exit(&conn, &card, &session, at("2026-11-25 20:00:00")).unwrap().1, 0);

        create_session(&mut conn, "v1", "AAAA0004", "entry_car", now, "p", &op).unwrap();
        let (card, session) = find_active_for_exit(&conn, "AAAA0004").unwrap();
        assert_eq!(compute_exit(&conn, &card, &session, at("2026-11-26 20:00:00")).unwrap().1, 0);
    }

    #[test]
    fn blocked_or_unknown_card_rejected() {
        let (conn, _) = setup();
        conn.execute("UPDATE cards SET status='lost' WHERE id='AAAA0001'", []).unwrap();
        assert!(validate_entry(&conn, "AAAA0001", at("2026-11-25 08:00:00")).is_err());
        assert!(validate_entry(&conn, "ZZZZ", at("2026-11-25 08:00:00")).is_err());
    }
}
