use super::check_lane;
use crate::domain::session;
use crate::db::repo::Card;
use crate::error::{AppError, Result};
use crate::state::{data_url, AppState, Pending};
use crate::util;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct EntryPreview {
    pub lane: String,
    pub card: Card,
    pub entry_time: String,
    pub photo: String,
}

/// Bước 1: quẹt thẻ -> kiểm tra + chụp ảnh. CHƯA ghi DB; chờ bảo vệ xác nhận.
#[tauri::command]
pub fn prepare_entry(state: State<'_, AppState>, card_code: String, lane: String) -> Result<EntryPreview> {
    state.require_user()?;
    check_lane(&lane, "entry_")?;

    let at = util::now();
    let card = session::validate_entry(&state.db(), &card_code, at)?;

    let jpeg = state.camera.capture_snapshot(&lane)?;
    let temp = state.write_temp("entry", &jpeg)?;
    state.set_pending(&lane, Pending::Entry { card: card.clone(), temp, at });

    Ok(EntryPreview { lane, card, entry_time: util::fmt(at), photo: data_url(&jpeg) })
}

/// Chụp lại ảnh cho thao tác đang chờ (F5). Trả về ảnh mới dạng data URL.
#[tauri::command]
pub fn recapture(state: State<'_, AppState>, lane: String) -> Result<String> {
    state.require_user()?;
    Ok(data_url(&state.recapture(&lane)?))
}

/// Bỏ thao tác đang chờ (Esc).
#[tauri::command]
pub fn cancel_pending(state: State<'_, AppState>, lane: String) -> Result<()> {
    state.require_user()?;
    state.discard_pending(&lane);
    Ok(())
}

/// Bước 2: bảo vệ xác nhận -> tạo phiên đỗ xe. Trả về session_id.
/// Phần mềm KHÔNG điều khiển barrier; bảo vệ tự bấm nút vật lý.
#[tauri::command]
pub fn confirm_entry(state: State<'_, AppState>, lane: String) -> Result<String> {
    let op = state.require_user()?;
    let (card, temp, at) = match state.take_pending(&lane) {
        Some(Pending::Entry { card, temp, at }) => (card, temp, at),
        Some(other) => {
            let _ = std::fs::remove_file(other.temp());
            return Err(AppError::msg("Không có xe vào đang chờ xác nhận trên làn này"));
        }
        None => return Err(AppError::msg("Không có xe vào đang chờ xác nhận trên làn này")),
    };

    let session_id = uuid::Uuid::new_v4().to_string();
    let rel = util::photo_rel(at, "entry", &session_id);
    let abs = state.data_dir.join(&rel);
    if let Some(dir) = abs.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::rename(&temp, &abs)?;

    let res = session::create_session(&mut state.db(), &session_id, &card.id, &lane, at, &rel, &op.id);
    if let Err(e) = res {
        let _ = std::fs::remove_file(&abs);
        return Err(e);
    }
    Ok(session_id)
}
