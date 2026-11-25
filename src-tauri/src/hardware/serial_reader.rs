use super::frame::FrameParser;
use std::io::{ErrorKind, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub enum ReaderEvent {
    /// Đầu đọc gửi một mã thẻ.
    Card(String),
    /// Trạng thái kết nối đổi (true = đã mở được cổng).
    Status(bool),
}

/// Callback nhận sự kiện: (hướng "entry"/"exit", sự kiện). Không phụ thuộc Tauri để test được.
pub type Emit = Arc<dyn Fn(&str, ReaderEvent) + Send + Sync>;

/// Giữ luồng đọc chạy; thả (drop) là dừng luồng.
pub struct ReaderHandle {
    stop: Arc<AtomicBool>,
}

impl Drop for ReaderHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// Quẹt lại cùng một thẻ trong khoảng này coi là một lần (đầu đọc thường lặp mã khi thẻ còn trong vùng đọc).
const DEBOUNCE: Duration = Duration::from_millis(1500);
const RETRY_EVERY: Duration = Duration::from_secs(2);

pub fn spawn(direction: &str, port: &str, baud: u32, emit: Emit) -> ReaderHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let (flag, direction, port) = (stop.clone(), direction.to_string(), port.to_string());
    thread::spawn(move || run(&direction, &port, baud, &emit, &flag));
    ReaderHandle { stop }
}

pub fn list_ports() -> Vec<String> {
    serialport::available_ports()
        .map(|ps| ps.into_iter().map(|p| p.port_name).collect())
        .unwrap_or_default()
}

fn run(direction: &str, port: &str, baud: u32, emit: &Emit, stop: &AtomicBool) {
    let mut connected = false;
    let mut set_status = |now: bool| {
        if now != connected {
            connected = now;
            emit(direction, ReaderEvent::Status(now));
        }
    };

    while !stop.load(Ordering::Relaxed) {
        match serialport::new(port, baud).timeout(Duration::from_millis(200)).open() {
            Ok(mut p) => {
                set_status(true);
                let mut parser = FrameParser::default();
                let mut buf = [0u8; 64];
                let mut last: Option<(String, Instant)> = None;
                while !stop.load(Ordering::Relaxed) {
                    match p.read(&mut buf) {
                        Ok(0) => break, // cổng đóng
                        Ok(n) => {
                            for code in parser.feed(&buf[..n]) {
                                let repeat = matches!(&last, Some((c, t)) if *c == code && t.elapsed() < DEBOUNCE);
                                last = Some((code.clone(), Instant::now()));
                                if !repeat {
                                    emit(direction, ReaderEvent::Card(code));
                                }
                            }
                        }
                        Err(e) if e.kind() == ErrorKind::TimedOut => {}
                        Err(_) => break, // rút cáp, v.v. -> thử mở lại
                    }
                }
                set_status(false);
            }
            Err(_) => set_status(false),
        }

        // Chờ rồi thử mở lại, nhưng vẫn phản ứng nhanh khi bị yêu cầu dừng.
        let until = Instant::now() + RETRY_EVERY;
        while Instant::now() < until && !stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn missing_port_reports_disconnected_then_stops_on_drop() {
        let events = Arc::new(Mutex::new(Vec::<String>::new()));
        let sink = events.clone();
        let emit: Emit = Arc::new(move |dir, ev| {
            let s = match ev {
                ReaderEvent::Card(c) => format!("{dir}:card:{c}"),
                ReaderEvent::Status(b) => format!("{dir}:status:{b}"),
            };
            sink.lock().unwrap().push(s);
        });
        let h = spawn("entry", "/dev/khong-ton-tai-xyz", 9600, emit);
        thread::sleep(Duration::from_millis(300));
        drop(h);
        // Cổng không mở được: chưa từng "connected", nên không có sự kiện nào (không spam).
        assert!(events.lock().unwrap().is_empty());
    }
}
