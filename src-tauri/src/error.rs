use serde::{Serialize, Serializer};

/// Lỗi trả về cho frontend dưới dạng chuỗi tiếng Việt (hiển thị thẳng cho bảo vệ).
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Msg(String),
    #[error("Phiên đăng nhập đã hết hạn, vui lòng đăng nhập lại")]
    Unauthenticated,
    #[error("Bạn không có quyền thực hiện thao tác này")]
    Forbidden,
    #[error("Lỗi cơ sở dữ liệu: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Lỗi tệp: {0}")]
    Io(#[from] std::io::Error),
    #[error("Lỗi hệ thống: {0}")]
    Other(String),
}

impl AppError {
    pub fn msg(s: impl Into<String>) -> Self {
        Self::Msg(s.into())
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
