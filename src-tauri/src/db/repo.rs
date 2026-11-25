use crate::domain::fee::Rule;
use crate::error::Result;
use chrono::NaiveDateTime;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Card {
    pub id: String,
    pub card_type: String,
    pub display_code: String,
    pub owner_name: Option<String>,
    pub owner_phone: Option<String>,
    pub vehicle_plate: Option<String>,
    pub vehicle_type: Option<String>,
    pub valid_to: Option<String>,
    pub status: String,
    pub notes: Option<String>,
}

const CARD_COLS: &str = "id, card_type, display_code, owner_name, owner_phone, vehicle_plate, \
                         vehicle_type, valid_to, status, notes";

impl Card {
    fn from_row(r: &Row) -> rusqlite::Result<Self> {
        Ok(Card {
            id: r.get(0)?,
            card_type: r.get(1)?,
            display_code: r.get(2)?,
            owner_name: r.get(3)?,
            owner_phone: r.get(4)?,
            vehicle_plate: r.get(5)?,
            vehicle_type: r.get(6)?,
            valid_to: r.get(7)?,
            status: r.get::<_, Option<String>>(8)?.unwrap_or_else(|| "active".into()),
            notes: r.get(9)?,
        })
    }
}

/// Tìm thẻ theo mã hex của đầu đọc, hoặc theo mã in trên thẻ (nhập tay khi đầu đọc hỏng).
pub fn find_card(conn: &Connection, code: &str) -> Result<Option<Card>> {
    let code = code.trim();
    let sql = format!(
        "SELECT {CARD_COLS} FROM cards WHERE upper(id) = upper(?1) OR display_code = ?1 \
         ORDER BY (upper(id) = upper(?1)) DESC LIMIT 1"
    );
    Ok(conn.query_row(&sql, params![code], Card::from_row).optional()?)
}

pub fn list_cards(conn: &Connection) -> Result<Vec<Card>> {
    let sql = format!("SELECT {CARD_COLS} FROM cards ORDER BY created_at DESC, id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], Card::from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn display_code_exists(conn: &Connection, code: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM cards WHERE display_code = ?1 OR upper(id) = upper(?1))",
        params![code],
        |r| r.get(0),
    )?)
}

#[derive(Debug, Clone)]
pub struct ActiveSession {
    pub id: String,
    pub card_id: String,
    pub lane: String,
    pub entry_time: NaiveDateTime,
    pub entry_photo_path: Option<String>,
}

pub fn find_active_session(conn: &Connection, card_id: &str) -> Result<Option<ActiveSession>> {
    Ok(conn
        .query_row(
            "SELECT id, card_id, lane, entry_time, entry_photo_path \
             FROM sessions WHERE card_id = ?1 AND status = 'active'",
            params![card_id],
            |r| {
                let entry: String = r.get(3)?;
                Ok(ActiveSession {
                    id: r.get(0)?,
                    card_id: r.get(1)?,
                    lane: r.get(2)?,
                    entry_time: crate::util::parse(&entry).unwrap_or_default(),
                    entry_photo_path: r.get(4)?,
                })
            },
        )
        .optional()?)
}

pub fn active_rules(conn: &Connection, vehicle_type: &str) -> Result<Vec<Rule>> {
    let mut stmt = conn.prepare(
        "SELECT minute_from, minute_to, price_per_unit, unit_minutes FROM pricing_rules \
         WHERE vehicle_type = ?1 AND is_active = 1 ORDER BY minute_from",
    )?;
    let rows = stmt.query_map(params![vehicle_type], |r| {
        Ok(Rule {
            minute_from: r.get(0)?,
            minute_to: r.get(1)?,
            price_per_unit: r.get(2)?,
            unit_minutes: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn get_config(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM app_config WHERE key = ?1", params![key], |r| r.get(0))
        .optional()?)
}

pub fn set_config(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO app_config (key, value, updated_at) VALUES (?1, ?2, ?3) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, crate::util::fmt(crate::util::now())],
    )?;
    Ok(())
}
