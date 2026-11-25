use crate::domain::auth::Operator;
use crate::db::{self, repo::ActiveSession, repo::Card};
use crate::error::{AppError, Result};
use crate::hardware::serial_reader::ReaderHandle;
use crate::hardware::{Camera, MockCamera};
use base64::Engine;
use chrono::NaiveDateTime;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// Một ca trực kéo dài tối đa 8 giờ (spec 12.2).
const SESSION_TTL: Duration = Duration::from_secs(8 * 60 * 60);

/// Thao tác đang chờ bảo vệ xác nhận, theo từng làn. Chưa ghi vào DB.
pub enum Pending {
    Entry {
        card: Card,
        temp: PathBuf,
        at: NaiveDateTime,
    },
    Exit {
        card: Card,
        session: ActiveSession,
        temp: PathBuf,
        at: NaiveDateTime,
        duration: i64,
        fee: i64,
    },
}

impl Pending {
    pub fn temp(&self) -> &Path {
        match self {
            Pending::Entry { temp, .. } | Pending::Exit { temp, .. } => temp,
        }
    }
}

pub struct AppState {
    db: Mutex<Connection>,
    pub data_dir: PathBuf,
    pub camera: Box<dyn Camera>,
    user: Mutex<Option<(Operator, Instant)>>,
    pending: Mutex<HashMap<String, Pending>>,
    readers: Mutex<HashMap<String, ReaderHandle>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl AppState {
    pub fn init(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(data_dir.join("photos/temp"))?;
        let conn = db::open(&data_dir.join("parking.db"))?;
        // Ảnh tạm còn sót từ lần chạy trước (app tắt giữa chừng) không còn ý nghĩa.
        if let Ok(rd) = std::fs::read_dir(data_dir.join("photos/temp")) {
            for e in rd.flatten() {
                let _ = std::fs::remove_file(e.path());
            }
        }
        Ok(Self {
            db: Mutex::new(conn),
            data_dir,
            camera: Box::new(MockCamera),
            user: Mutex::new(None),
            pending: Mutex::new(HashMap::new()),
            readers: Mutex::new(HashMap::new()),
        })
    }

    /// Thay đầu đọc của một hướng ("entry"/"exit"). Đầu đọc cũ bị thả -> luồng đọc tự dừng.
    pub fn set_reader(&self, direction: &str, handle: Option<ReaderHandle>) {
        let mut m = lock(&self.readers);
        match handle {
            Some(h) => m.insert(direction.to_string(), h),
            None => m.remove(direction),
        };
    }

    pub fn db(&self) -> MutexGuard<'_, Connection> {
        lock(&self.db)
    }

    pub fn set_user(&self, op: Option<Operator>) {
        *lock(&self.user) = op.map(|o| (o, Instant::now()));
    }

    /// Người đang đăng nhập, hoặc None nếu chưa đăng nhập / hết hạn ca.
    pub fn current_user(&self) -> Option<Operator> {
        let mut u = lock(&self.user);
        match &*u {
            Some((op, at)) if at.elapsed() < SESSION_TTL => Some(op.clone()),
            Some(_) => {
                *u = None;
                None
            }
            None => None,
        }
    }

    pub fn require_user(&self) -> Result<Operator> {
        self.current_user().ok_or(AppError::Unauthenticated)
    }

    pub fn require_manager(&self) -> Result<Operator> {
        let op = self.require_user()?;
        if op.role == "manager" { Ok(op) } else { Err(AppError::Forbidden) }
    }

    /// Lưu ảnh tạm, trả về đường dẫn tuyệt đối.
    pub fn write_temp(&self, kind: &str, bytes: &[u8]) -> Result<PathBuf> {
        let path = self
            .data_dir
            .join("photos/temp")
            .join(format!("{kind}_{}.jpg", uuid::Uuid::new_v4()));
        std::fs::write(&path, bytes)?;
        Ok(path)
    }

    /// Đặt thao tác chờ cho làn; thao tác cũ của làn đó (nếu có) bị hủy và xóa ảnh tạm.
    pub fn set_pending(&self, lane: &str, p: Pending) {
        if let Some(old) = lock(&self.pending).insert(lane.to_string(), p) {
            let _ = std::fs::remove_file(old.temp());
        }
    }

    pub fn take_pending(&self, lane: &str) -> Option<Pending> {
        lock(&self.pending).remove(lane)
    }

    /// Hủy thao tác chờ và xóa ảnh tạm.
    pub fn discard_pending(&self, lane: &str) {
        if let Some(p) = self.take_pending(lane) {
            let _ = std::fs::remove_file(p.temp());
        }
    }

    /// Chụp lại ảnh cho thao tác đang chờ, ghi đè ảnh tạm, trả về byte ảnh mới.
    pub fn recapture(&self, lane: &str) -> Result<Vec<u8>> {
        let temp = lock(&self.pending)
            .get(lane)
            .map(|p| p.temp().to_path_buf())
            .ok_or_else(|| AppError::msg("Không có thao tác nào đang chờ trên làn này"))?;
        let bytes = self.camera.capture_snapshot(lane)?;
        std::fs::write(&temp, &bytes)?;
        Ok(bytes)
    }
}

pub fn data_url(jpeg: &[u8]) -> String {
    format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(jpeg)
    )
}
