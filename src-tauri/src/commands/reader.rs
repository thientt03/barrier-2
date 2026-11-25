use crate::db::repo;
use crate::domain::audit;
use crate::error::{AppError, Result};
use crate::hardware::serial_reader::{self, Emit, ReaderEvent};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

const BAUDS: [u32; 6] = [2400, 4800, 9600, 19200, 38400, 115200];
const DEFAULT_BAUD: u32 = 9600;

fn port_key(direction: &str) -> String {
    format!("card_reader_{direction}_port")
}

#[derive(Serialize, Deserialize)]
pub struct ReaderConfig {
    /// Rỗng = không dùng cổng serial (đầu đọc USB HID giả lập bàn phím thì không cần cấu hình).
    pub entry_port: String,
    pub exit_port: String,
    pub baud: u32,
}

fn read_config(state: &AppState) -> Result<ReaderConfig> {
    let conn = state.db();
    let get = |k: &str| -> Result<String> { Ok(repo::get_config(&conn, k)?.unwrap_or_default()) };
    let baud = repo::get_config(&conn, "card_reader_baud")?
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_BAUD);
    Ok(ReaderConfig { entry_port: get(&port_key("entry"))?, exit_port: get(&port_key("exit"))?, baud })
}

/// Mở lại các đầu đọc theo cấu hình hiện tại. Sự kiện gửi sang giao diện:
/// `card-tapped` {direction, card_id} và `reader-status` {direction, connected}.
pub fn restart_readers(app: &AppHandle, state: &AppState) -> Result<()> {
    let cfg = read_config(state)?;
    let handle = app.clone();
    let emit: Emit = Arc::new(move |direction, ev| {
        let _ = match ev {
            ReaderEvent::Card(card_id) => handle
                .emit("card-tapped", serde_json::json!({ "direction": direction, "card_id": card_id })),
            ReaderEvent::Status(connected) => handle
                .emit("reader-status", serde_json::json!({ "direction": direction, "connected": connected })),
        };
    });
    for (direction, port) in [("entry", &cfg.entry_port), ("exit", &cfg.exit_port)] {
        let reader = (!port.is_empty()).then(|| serial_reader::spawn(direction, port, cfg.baud, emit.clone()));
        state.set_reader(direction, reader);
    }
    Ok(())
}

/// Gọi lúc khởi động app. Lỗi cấu hình không được làm app không mở được.
pub fn start_on_boot(app: &AppHandle) {
    let state = app.state::<AppState>();
    if let Err(e) = restart_readers(app, &state) {
        eprintln!("không khởi động được đầu đọc thẻ: {e}");
    }
}

#[tauri::command]
pub fn list_serial_ports(state: State<'_, AppState>) -> Result<Vec<String>> {
    state.require_manager()?;
    Ok(serial_reader::list_ports())
}

#[tauri::command]
pub fn get_reader_config(state: State<'_, AppState>) -> Result<ReaderConfig> {
    state.require_manager()?;
    read_config(&state)
}

#[tauri::command]
pub fn set_reader_config(app: AppHandle, state: State<'_, AppState>, input: ReaderConfig) -> Result<()> {
    let op = state.require_manager()?;
    if !BAUDS.contains(&input.baud) {
        return Err(AppError::msg("Tốc độ baud không hợp lệ"));
    }
    let (entry, exit) = (input.entry_port.trim(), input.exit_port.trim());
    if !entry.is_empty() && entry == exit {
        return Err(AppError::msg("Hai làn không thể dùng chung một cổng"));
    }
    {
        let conn = state.db();
        repo::set_config(&conn, &port_key("entry"), entry)?;
        repo::set_config(&conn, &port_key("exit"), exit)?;
        repo::set_config(&conn, "card_reader_baud", &input.baud.to_string())?;
        audit::log(
            &conn,
            None,
            Some(&op.id),
            "config_change",
            Some(serde_json::json!({ "entry_port": entry, "exit_port": exit, "baud": input.baud })),
        )?;
    }
    restart_readers(&app, &state)
}

