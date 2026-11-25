/// Tách mã thẻ từ luồng byte của đầu đọc RS232/USB-serial.
///
/// Hỗ trợ cả hai kiểu phổ biến của đầu đọc 125kHz:
/// - mỗi mã kết thúc bằng CR và/hoặc LF ("A1B2C3D4\r\n");
/// - khung STX..ETX (0x02 "mã" 0x03).
/// Chỉ giữ ký tự in được; mã được viết hoa. Byte rác bị bỏ qua.
#[derive(Default)]
pub struct FrameParser {
    buf: Vec<u8>,
}

const MAX_CODE_LEN: usize = 64;

impl FrameParser {
    pub fn feed(&mut self, data: &[u8]) -> Vec<String> {
        let mut out = Vec::new();
        for &b in data {
            match b {
                b'\r' | b'\n' | 0x03 => {
                    if let Some(code) = self.take() {
                        out.push(code);
                    }
                }
                0x02 => self.buf.clear(),
                0x20..=0x7e => {
                    // Quá dài = rác (đứt khung); bỏ và bắt đầu lại.
                    if self.buf.len() >= MAX_CODE_LEN {
                        self.buf.clear();
                    }
                    self.buf.push(b);
                }
                _ => {}
            }
        }
        out
    }

    fn take(&mut self) -> Option<String> {
        let s = String::from_utf8_lossy(&self.buf).trim().to_uppercase();
        self.buf.clear();
        if s.is_empty() { None } else { Some(s) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_terminated_codes() {
        let mut p = FrameParser::default();
        assert_eq!(p.feed(b"a1b2c3d4\r\n"), vec!["A1B2C3D4"]);
        assert_eq!(p.feed(b"00112233\n"), vec!["00112233"]);
    }

    #[test]
    fn code_split_across_reads() {
        let mut p = FrameParser::default();
        assert!(p.feed(b"A1B2").is_empty());
        assert_eq!(p.feed(b"C3D4\r"), vec!["A1B2C3D4"]);
    }

    #[test]
    fn stx_etx_framing_and_empty_lines() {
        let mut p = FrameParser::default();
        assert_eq!(p.feed(b"\x02AABBCCDD11\x03\r\n\r\n"), vec!["AABBCCDD11"]);
    }

    #[test]
    fn two_codes_in_one_read() {
        let mut p = FrameParser::default();
        assert_eq!(p.feed(b"AAAA\nBBBB\n"), vec!["AAAA", "BBBB"]);
    }

    #[test]
    fn garbage_and_overlong_input_do_not_grow_unbounded() {
        let mut p = FrameParser::default();
        let junk = vec![b'X'; 1000];
        assert!(p.feed(&junk).is_empty());
        assert!(p.feed(&[0x00, 0xff, 0x80]).is_empty());
        assert_eq!(p.feed(b"OK\n").len(), 1);
    }
}
