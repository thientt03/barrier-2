use super::check_lane;
use crate::domain::{audit, session};
use crate::db::repo::Card;
use crate::error::{AppError, Result};
use crate::state::{data_url, AppState, Pending};
use crate::util;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct ExitPreview {
    pub lane: String,
    pub card: Card,
    pub entry_time: String,
    pub exit_time: String,
    pub duration_minutes: i64,
    pub fee: i64,
    pub is_free: bool,
    pub entry_photo: Option<String>,
    pub exit_photo: String,
}

/// Bước 1: quẹt thẻ ra -> tìm phiên, chụp ảnh, tính phí. CHƯA ghi DB.
#[tauri::command]
pub fn prepare_exit(state: State<'_, AppState>, card_code: String, lane: String) -> Result<ExitPreview> {
    state.require_user()?;
    check_lane(&lane, "exit_")?;

    let at = util::now();
    let (card, sess, duration, fee) = {
        let conn = state.db();
        let (card, sess) = session::find_active_for_exit(&conn, &card_code)?;
        let (duration, fee) = session::compute_exit(&conn, &card, &sess, at)?;
        (card, sess, duration, fee)
    };

    // Ảnh lúc vào có thể thiếu (đã bị xóa tay); không chặn việc cho xe ra.
    let entry_photo = sess
        .entry_photo_path
        .as_ref()
        .and_then(|rel| std::fs::read(state.data_dir.join(rel)).ok())
        .map(|b| data_url(&b));

    let jpeg = state.camera.capture_snapshot(&lane)?;
    let temp = state.write_temp("exit", &jpeg)?;

    let preview = ExitPreview {
        lane: lane.clone(),
        card: card.clone(),
        entry_time: util::fmt(sess.entry_time),
        exit_time: util::fmt(at),
        duration_minutes: duration,
        fee,
        is_free: fee == 0,
        entry_photo,
        exit_photo: data_url(&jpeg),
    };
    state.set_pending(&lane, Pending::Exit { card, session: sess, temp, at, duration, fee });
    Ok(preview)
}

/// Bước 2: bảo vệ bấm "ĐÃ THU TIỀN" -> đóng phiên. Trả về phí đã thu.
#[tauri::command]
pub fn confirm_exit(state: State<'_, AppState>, lane: String) -> Result<i64> {
    let op = state.require_user()?;
    let (sess, temp, at, duration, fee) = match state.take_pending(&lane) {
        Some(Pending::Exit { session, temp, at, duration, fee, .. }) => (session, temp, at, duration, fee),
        Some(other) => {
            let _ = std::fs::remove_file(other.temp());
            return Err(AppError::msg("Không có xe ra đang chờ xác nhận trên làn này"));
        }
        None => return Err(AppError::msg("Không có xe ra đang chờ xác nhận trên làn này")),
    };

    let rel = util::photo_rel(at, "exit", &sess.id);
    let abs = state.data_dir.join(&rel);
    if let Some(dir) = abs.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::rename(&temp, &abs)?;

    let res = session::complete_session(&mut state.db(), &sess.id, at, &rel, &op.id, duration, fee);
    if let Err(e) = res {
        let _ = std::fs::remove_file(&abs);
        return Err(e);
    }
    Ok(fee)
}

/// Bảo vệ từ chối cho xe ra (khách không trả tiền...). Phiên vẫn đang đỗ; ghi audit `refuse_exit`.
#[tauri::command]
pub fn refuse_exit(state: State<'_, AppState>, lane: String) -> Result<()> {
    let op = state.require_user()?;
    let (sess_id, card_id, fee) = match state.take_pending(&lane) {
        Some(Pending::Exit { session, card, temp, fee, .. }) => {
            let _ = std::fs::remove_file(&temp);
            (session.id, card.id, fee)
        }
        Some(other) => {
            let _ = std::fs::remove_file(other.temp());
            return Err(AppError::msg("Không có xe ra đang chờ trên làn này"));
        }
        None => return Err(AppError::msg("Không có xe ra đang chờ trên làn này")),
    };
    audit::log(
        &state.db(),
        Some(&sess_id),
        Some(&op.id),
        "refuse_exit",
        Some(serde_json::json!({ "card_id": card_id, "fee": fee })),
    )
}
