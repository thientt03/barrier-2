pub mod frame;
pub mod serial_reader;

use crate::error::{AppError, Result};
use image::{codecs::jpeg::JpegEncoder, ExtendedColorType};

/// Giao diện camera. Bản Dahua (RTSP/HTTP snapshot) sẽ cài đặt trait này ở bước sau.
pub trait Camera: Send + Sync {
    /// Chụp một khung hình, trả về byte JPEG.
    fn capture_snapshot(&self, lane: &str) -> Result<Vec<u8>>;
}

/// Camera giả lập: mỗi lần chụp ra một ảnh khác nhau để dev/test không cần phần cứng.
pub struct MockCamera;

impl Camera for MockCamera {
    fn capture_snapshot(&self, _lane: &str) -> Result<Vec<u8>> {
        let (w, h) = (640u32, 360u32);
        let seed = (chrono::Local::now().timestamp_millis() & 0xFFFF) as u32;
        let mut raw = Vec::with_capacity((w * h * 3) as usize);
        for y in 0..h {
            for x in 0..w {
                let in_car = x > 200 && x < 440 && y > 150 && y < 290;
                let px = if in_car {
                    [(seed % 200 + 30) as u8, 60, 90]
                } else {
                    [(x * 255 / w) as u8, (y * 255 / h) as u8, ((seed >> 4) % 255) as u8]
                };
                raw.extend_from_slice(&px);
            }
        }
        let mut out = Vec::new();
        JpegEncoder::new_with_quality(&mut out, 80)
            .encode(&raw, w, h, ExtendedColorType::Rgb8)
            .map_err(|e| AppError::Other(e.to_string()))?;
        Ok(out)
    }
}
