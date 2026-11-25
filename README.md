# Quản lý bãi đỗ xe

Cài đặt theo `SPEC Hệ thống quản lý.md`. Phạm vi hiện tại: Milestone 1 + 2 (luồng vào/ra end-to-end). Camera đang là bản giả lập (`MockCamera`).

## Chạy thử

Cần Rust (rustup), Node.js, và các thư viện hệ thống của Tauri 2 (xem https://tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri dev      # chạy app
npm run tauri build    # đóng gói installer
cd src-tauri && cargo test   # unit test (tính phí, auth, luồng vào/ra)
```

Lần mở đầu tiên app yêu cầu tạo tài khoản quản lý. Vào mục **Thẻ** để thêm thẻ, rồi thử quẹt ở **Xe vào** / **Xe ra**.
Đầu đọc RFID dạng USB HID gõ mã thẻ vào ô nhập rồi gửi Enter; cũng có thể gõ tay mã thẻ (F2).

## Phím tắt
`Enter` xác nhận · `Esc` hủy/từ chối · `F2` nhập mã thẻ · `F5` chụp lại ảnh
