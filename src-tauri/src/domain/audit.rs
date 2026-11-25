use crate::error::Result;
use crate::util;
use rusqlite::{params, Connection};

/// Ghi nhật ký thao tác. Gọi trong cùng transaction với thao tác chính để không bao giờ lệch.
pub fn log(
    conn: &Connection,
    session_id: Option<&str>,
    operator_id: Option<&str>,
    action: &str,
    metadata: Option<serde_json::Value>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO audit_logs (session_id, operator_id, action, metadata, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            session_id,
            operator_id,
            action,
            metadata.map(|m| m.to_string()),
            util::fmt(util::now())
        ],
    )?;
    Ok(())
}
