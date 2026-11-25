use crate::domain::audit;
use crate::db::repo::{self, Card};
use crate::error::{AppError, Result};
use crate::state::AppState;
use crate::util;
use rusqlite::params;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateCardInput {
    pub id: String,
    pub card_type: String,
    pub display_code: String,
    pub owner_name: Option<String>,
    pub owner_phone: Option<String>,
    pub vehicle_plate: Option<String>,
    pub vehicle_type: Option<String>,
    pub valid_to: Option<String>,
}

/// Chuỗi rỗng / toàn khoảng trắng -> None.
fn clean(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

#[tauri::command]
pub fn list_cards(state: State<'_, AppState>) -> Result<Vec<Card>> {
    state.require_manager()?;
    repo::list_cards(&state.db())
}

#[tauri::command]
pub fn create_card(state: State<'_, AppState>, input: CreateCardInput) -> Result<Card> {
    let op = state.require_manager()?;

    let id = input.id.trim().to_uppercase();
    let display_code = input.display_code.trim().to_string();
    if id.is_empty() || display_code.is_empty() {
        return Err(AppError::msg("Vui lòng nhập mã thẻ (hex) và mã in trên thẻ"));
    }
    if !["monthly", "guest", "vip"].contains(&input.card_type.as_str()) {
        return Err(AppError::msg("Loại thẻ không hợp lệ"));
    }
    let vehicle_type = clean(input.vehicle_type);
    if let Some(v) = &vehicle_type {
        if !["motorbike", "car", "truck"].contains(&v.as_str()) {
            return Err(AppError::msg("Loại xe không hợp lệ"));
        }
    }
    if input.card_type == "monthly" && vehicle_type.is_none() {
        return Err(AppError::msg("Vé tháng cần chọn loại xe"));
    }
    let valid_to = match clean(input.valid_to) {
        Some(s) => Some(
            util::parse(&s)
                .map(util::fmt)
                .ok_or_else(|| AppError::msg("Ngày hết hạn không hợp lệ"))?,
        ),
        None => None,
    };

    let mut conn = state.db();
    if repo::display_code_exists(&conn, &id)? || repo::display_code_exists(&conn, &display_code)? {
        return Err(AppError::msg("Mã thẻ hoặc mã in trên thẻ đã tồn tại"));
    }
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO cards (id, card_type, display_code, owner_name, owner_phone, vehicle_plate, \
         vehicle_type, valid_to) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            id,
            input.card_type,
            display_code,
            clean(input.owner_name),
            clean(input.owner_phone),
            clean(input.vehicle_plate),
            vehicle_type,
            valid_to
        ],
    )?;
    audit::log(
        &tx,
        None,
        Some(&op.id),
        "card_create",
        Some(serde_json::json!({ "card_id": id, "card_type": input.card_type })),
    )?;
    tx.commit()?;
    repo::find_card(&conn, &id)?.ok_or_else(|| AppError::Other("không đọc lại được thẻ vừa tạo".into()))
}
