pub mod repo;

use crate::error::Result;
use rusqlite::Connection;
use std::path::Path;

const SCHEMA_V1: &str = include_str!("schema.sql");

/// Mở DB tại `path`, bật WAL + foreign keys, chạy migration.
pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<()> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version < 1 {
        conn.execute_batch(SCHEMA_V1)?;
    }
    Ok(())
}

#[cfg(test)]
pub fn open_memory() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    migrate(&conn).unwrap();
    conn
}
