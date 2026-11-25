use crate::domain::audit;
use crate::error::{AppError, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

const BCRYPT_COST: u32 = 12;
const MIN_PASSWORD_LEN: usize = 8;

#[derive(Debug, Clone, Serialize)]
pub struct Operator {
    pub id: String,
    pub username: String,
    pub full_name: String,
    pub role: String,
}

pub fn has_any_operator(conn: &Connection) -> Result<bool> {
    Ok(conn.query_row("SELECT EXISTS(SELECT 1 FROM operators)", [], |r| r.get(0))?)
}

/// Tạo manager đầu tiên. Chỉ thành công khi chưa có operator nào.
pub fn create_first_manager(
    conn: &mut Connection,
    username: &str,
    password: &str,
    full_name: &str,
) -> Result<Operator> {
    let username = username.trim();
    let full_name = full_name.trim();
    if username.is_empty() || full_name.is_empty() {
        return Err(AppError::msg("Vui lòng nhập tên đăng nhập và họ tên"));
    }
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(AppError::msg(format!("Mật khẩu tối thiểu {MIN_PASSWORD_LEN} ký tự")));
    }
    let hash = bcrypt::hash(password, BCRYPT_COST).map_err(|e| AppError::Other(e.to_string()))?;
    let id = uuid::Uuid::new_v4().to_string();

    let tx = conn.transaction()?;
    if has_any_operator(&tx)? {
        return Err(AppError::msg("Hệ thống đã được thiết lập"));
    }
    tx.execute(
        "INSERT INTO operators (id, username, password_hash, full_name, role) \
         VALUES (?1, ?2, ?3, ?4, 'manager')",
        params![id, username, hash, full_name],
    )?;
    audit::log(
        &tx,
        None,
        Some(&id),
        "setup",
        Some(serde_json::json!({ "username": username })),
    )?;
    tx.commit()?;
    Ok(Operator { id, username: username.into(), full_name: full_name.into(), role: "manager".into() })
}

/// Xác thực. Ghi audit `login` / `login_failed`. Thông báo lỗi không tiết lộ tên đăng nhập có tồn tại hay không.
pub fn verify_login(conn: &Connection, username: &str, password: &str) -> Result<Operator> {
    let username = username.trim();
    let row: Option<(Operator, String)> = conn
        .query_row(
            "SELECT id, username, full_name, role, password_hash FROM operators \
             WHERE username = ?1 AND is_active = 1",
            params![username],
            |r| {
                Ok((
                    Operator { id: r.get(0)?, username: r.get(1)?, full_name: r.get(2)?, role: r.get(3)? },
                    r.get::<_, String>(4)?,
                ))
            },
        )
        .optional()?;

    // Luôn tốn một lần bcrypt (kể cả khi user không tồn tại) để thời gian phản hồi không lộ thông tin.
    let ok = match &row {
        Some((_, hash)) => bcrypt::verify(password, hash).unwrap_or(false),
        None => {
            let _ = bcrypt::hash(password, BCRYPT_COST);
            false
        }
    };

    match row {
        Some((op, _)) if ok => {
            audit::log(conn, None, Some(&op.id), "login", None)?;
            Ok(op)
        }
        _ => {
            audit::log(
                conn,
                None,
                None,
                "login_failed",
                Some(serde_json::json!({ "username": username })),
            )?;
            Err(AppError::msg("Sai tên đăng nhập hoặc mật khẩu"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_memory;

    #[test]
    fn setup_then_login() {
        let mut conn = open_memory();
        assert!(!has_any_operator(&conn).unwrap());
        create_first_manager(&mut conn, "admin", "matkhau123", "Quản lý").unwrap();
        assert!(has_any_operator(&conn).unwrap());

        assert_eq!(verify_login(&conn, "admin", "matkhau123").unwrap().role, "manager");
        assert!(verify_login(&conn, "admin", "sai").is_err());
        assert!(verify_login(&conn, "khongco", "matkhau123").is_err());

        let failed: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_logs WHERE action='login_failed'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(failed, 2);
    }

    #[test]
    fn setup_only_once_and_rejects_short_password() {
        let mut conn = open_memory();
        assert!(create_first_manager(&mut conn, "admin", "ngan", "A").is_err());
        create_first_manager(&mut conn, "admin", "matkhau123", "A").unwrap();
        assert!(create_first_manager(&mut conn, "admin2", "matkhau123", "B").is_err());
    }
}
