pub mod auth;
pub mod cards;
pub mod entry;
pub mod exit;
pub mod reader;

use crate::error::{AppError, Result};

/// Làn hợp lệ: entry_car, entry_bike, exit_car, exit_bike.
pub(crate) fn check_lane(lane: &str, prefix: &str) -> Result<()> {
    match lane.strip_prefix(prefix) {
        Some("car") | Some("bike") => Ok(()),
        _ => Err(AppError::msg("Làn không hợp lệ")),
    }
}
