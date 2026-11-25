# 📘 SPEC: Hệ thống quản lý bãi đỗ xe tự phát triển

**Phiên bản:** 1.0 — Bản đặc tả kỹ thuật
**Ngày:** 25/11/2026
**Tác giả:** [Bạn]
**Trạng thái:** Draft — sẵn sàng triển khai

---

## 📑 Mục lục

1. [Tổng quan dự án](#1-tổng-quan-dự-án)
2. [Mục tiêu và phạm vi](#2-mục-tiêu-và-phạm-vi)
3. [Kiến trúc hệ thống](#3-kiến-trúc-hệ-thống)
4. [Danh sách phần cứng](#4-danh-sách-phần-cứng)
5. [Stack công nghệ](#5-stack-công-nghệ)
6. [Mô hình dữ liệu](#6-mô-hình-dữ-liệu)
7. [Luồng nghiệp vụ](#7-luồng-nghiệp-vụ)
8. [Thiết kế UI](#8-thiết-kế-ui)
9. [Cấu trúc mã nguồn](#9-cấu-trúc-mã-nguồn)
10. [License management](#10-license-management)
11. [Xử lý sự cố & offline](#11-xử-lý-sự-cố--offline)
12. [Bảo mật & phân quyền](#12-bảo-mật--phân-quyền)
13. [Lộ trình phát triển](#13-lộ-trình-phát-triển)
14. [Deployment & vận hành](#14-deployment--vận-hành)
15. [Rủi ro & giảm thiểu](#15-rủi-ro--giảm-thiểu)

---

## 1. Tổng quan dự án

### 1.1 Bối cảnh

Xây dựng phần mềm quản lý bãi đỗ xe **on-premise** cho các tòa nhà (chung cư, văn phòng, bãi xe tư nhân), với đặc điểm:

- Có **bảo vệ trực 24/7** tại bốt
- **Không cần AI nhận diện biển số** — bảo vệ tự so ảnh bằng mắt
- **Barrier hoàn toàn độc lập** — bảo vệ bấm nút vật lý để mở
- **Thẻ RFID** là "chìa khóa" liên kết xe vào-ra
- **Desktop app** phân phối dưới dạng file `.exe`
- **Mô hình kinh doanh:** bán license theo tòa nhà, quản lý tập trung qua license server

### 1.2 Triết lý thiết kế

> **"Phần mềm là trợ lý, con người là người quyết định."**

- Phần mềm **không bao giờ** điều khiển barrier trực tiếp
- Phần mềm **không bao giờ** tự động ra quyết định thay bảo vệ
- Mọi quyết định quan trọng (mở barrier, thu tiền) đều do **bảo vệ** thực hiện
- Phần mềm chỉ: **ghi nhận, tính toán, hiển thị, lưu trữ**

### 1.3 Nguyên tắc kiến trúc

| Nguyên tắc | Ý nghĩa |
|---|---|
| **Local-first** | Toàn bộ hoạt động thiết yếu chạy trên máy local, không phụ thuộc cloud |
| **Decoupled systems** | Phần mềm và barrier độc lập hoàn toàn, không có dây tín hiệu |
| **Human-in-the-loop** | Bảo vệ là cầu nối duy nhất giữa phần mềm và barrier |
| **Fail-safe** | Khi mọi thứ hỏng, vẫn có fallback thủ công (nút bấm, sổ giấy) |
| **Offline-tolerant** | Mất mạng vẫn chạy bình thường, sync khi có mạng lại |

---

## 2. Mục tiêu và phạm vi

### 2.1 Mục tiêu

| # | Mục tiêu | Chỉ số đo |
|---|---|---|
| M1 | Số hóa quy trình bãi xe | Không còn dùng sổ giấy trong vận hành thường ngày |
| M2 | Tính phí tự động chính xác | Sai sót < 0.1% |
| M3 | Chống gian lận | 100% thao tác mở barrier có audit log |
| M4 | Chi phí thấp | Tổng phần cứng < 30 triệu/làn |
| M5 | Dễ deploy | Cài đặt < 15 phút/làn |
| M6 | Hỗ trợ multi-tenant | 1 vendor quản lý N tòa nhà qua license server |

### 2.2 Phạm vi

**Trong phạm vi:**
- Quản lý phiên đỗ xe (entry/exit)
- Quản lý thẻ RFID (vé tháng, khách vãng lai, VIP)
- Tính phí theo bảng giá cấu hình
- In vé + hóa đơn
- Báo cáo doanh thu, lưu lượng
- License management cho vendor
- Xử lý sự cố (mất thẻ, mất mạng, mất điện)

**Ngoài phạm vi (không làm trong v1.0):**
- AI/OCR nhận diện biển số
- App mobile cho khách hàng
- Thanh toán online (chỉ thu tiền mặt)
- Multi-tenant cloud (mỗi khách dùng bản local)
- Tích hợp camera an ninh bên ngoài
- Đặt chỗ trước

### 2.3 Đối tượng người dùng

| Vai trò | Số lượng/tòa nhà | Sử dụng |
|---|---|---|
| **Bảo vệ** | 1-4 | Màn hình bốt, xử lý entry/exit |
| **Quản lý** | 1 | Xem báo cáo, quản lý thẻ, bảng giá |
| **Vendor admin** | 1 (tổng) | License server, quản lý khách hàng |

---

## 3. Kiến trúc hệ thống

### 3.1 Sơ đồ tổng thể

```
╔═══════════════════════════════════════════════════════════════╗
║  HỆ THỐNG PHẦN MỀM (do bạn phát triển)                       ║
║                                                               ║
║   ┌──────────┐   ┌──────────┐   ┌──────────────────┐         ║
║   │ Camera   │   │ Đầu đọc  │   │  MINI PC         │         ║
║   │ Dahua    │──→│ thẻ RFID │──→│  (Tauri app)     │         ║
║   │ ×2       │   │ ×2       │   │                  │         ║
║   └──────────┘   └──────────┘   │  - Rust backend  │         ║
║                                  │  - Vue frontend  │         ║
║   ┌──────────┐                   │  - SQLite        │         ║
║   │ Thẻ RFID │                   │  - Lưu ảnh       │         ║
║   │ ×N       │                   └────────┬─────────┘         ║
║   └──────────┘                            │                   ║
║                                  ┌────────▼─────────┐         ║
║                                  │  MÀN HÌNH        │         ║
║                                  │  Bảo vệ          │         ║
║                                  └────────┬─────────┘         ║
╚═══════════════════════════════════════════│═══════════════════╝
                                            │
                                       👤 BẢO VỆ
                                    (cầu nối duy nhất)
                                            │
                                    👆 Bấm nút vật lý
                                            │
╔═══════════════════════════════════════════▼═══════════════════╗
║  HỆ THỐNG BARRIER (hoàn toàn độc lập)                        ║
║                                                               ║
║   ┌────────────────────────────────────────────┐             ║
║   │  Barrier Bisen BS-306                      │             ║
║   │  - Nút bấm trên bàn                        │             ║
║   │  - Remote RF (dự phòng)                    │             ║
║   │  - Tay quay khẩn cấp                       │             ║
║   │  ❌ Không kết nối với máy tính             │             ║
║   └────────────────────────────────────────────┘             ║
╚═══════════════════════════════════════════════════════════════╝

        ╔══════════════════════════════════════════╗
        ║  LICENSE SERVER (cloud, do vendor chạy)  ║
        ║  - Xác thực license key                  ║
        ║  - Khóa/mở tài khoản                     ║
        ║  - Phân phối update                      ║
        ╚══════════════════════════════════════════╝
                 ▲ phone-home mỗi 24h
                 │
        ┌────────┴─────────┐
        │ Mini PC (app)    │
        └──────────────────┘
```

### 3.2 Các thành phần

| Thành phần | Vai trò | Công nghệ |
|---|---|---|
| **Tauri app** | Phần mềm chính chạy tại bốt | Rust + Vue |
| **Rust backend** | Xử lý logic, đọc USB, chụp RTSP, DB | Rust |
| **Vue frontend** | UI cho bảo vệ và manager | Vue 3 + Vite |
| **SQLite** | Lưu dữ liệu local | `rusqlite` |
| **File storage** | Lưu ảnh vào/ra | Filesystem local |
| **License server** | Quản lý license đa khách hàng | FastAPI + PostgreSQL (cloud) |
| **Camera Dahua** | Chụp ảnh xe | RTSP + HTTP snapshot |
| **Đầu đọc RFID** | Đọc mã thẻ | USB HID hoặc RS232 |
| **Barrier BS-306** | Chắn xe | Điều khiển bằng nút bấm vật lý |

---

## 4. Danh sách phần cứng

### 4.1 BOM (Bill of Materials) — Mỗi làn

| # | Thiết bị | Model cụ thể | SL | Giá (VNĐ) | Ghi chú |
|---|---|---|---|---|---|
| 1 | Camera LPR | **Dahua IPC-HFW2239S-SA-LED-S2** | 1 | 2.500.000 – 3.000.000 | Ống kính **3.6mm** |
| 2 | Đầu đọc thẻ | RFID USB HID 125kHz (EM4100) | 1 | 300.000 – 500.000 | Cắm USB, không cần driver |
| 3 | Thẻ RFID | EM4100 125kHz | 50–200 | 10.000 – 20.000/thẻ | In logo nếu cần |
| 4 | Barrier | **Bisen BS-306** DC 24V | 1 | 11.000.000 – 13.000.000 | Có nút OPEN trên bo mạch |
| 5 | Remote RF | Đi kèm barrier | 1 | (miễn phí) | Dự phòng khi mất mạng |
| 6 | Nút bấm vật lý | Nút nhấn công nghiệp | 1 | 100.000 – 300.000 | Đấu vào cổng OPEN của barrier |

### 4.2 Thiết bị dùng chung (1 bộ cho cả bãi)

| # | Thiết bị | Model gợi ý | SL | Giá (VNĐ) | Ghi chú |
|---|---|---|---|---|---|
| 7 | Mini PC | Intel N100/N200, 8GB RAM, 256GB SSD | 1 | 3.500.000 – 5.000.000 | Cần ít nhất 2 cổng USB |
| 8 | Màn hình | 21-24" hoặc tablet | 1 | 2.000.000 – 4.000.000 | Cho bảo vệ |
| 9 | Switch PoE | 8-port PoE+ | 1 | 800.000 – 1.500.000 | Cấp nguồn camera |
| 10 | Dây mạng Cat6 | Cuộn 100m | 1 | 500.000 – 800.000 | Nối camera |
| 11 | Máy in nhiệt | 80mm, USB | 1 | 1.500.000 – 2.000.000 | In vé + hóa đơn (tùy chọn) |
| 12 | UPS | 600–1000VA | 1 | 1.000.000 – 2.000.000 | Bảo vệ mini PC khi mất điện (tùy chọn) |

### 4.3 Tổng chi phí

| Hạng mục | Chi phí |
|---|---|
| Phần cứng/làn (mục 1-6) | **~14 – 17 triệu** |
| Phần cứng dùng chung (mục 7-12) | **~9 – 15 triệu** |
| **Tổng cho 1 làn + bộ chung** | **~23 – 32 triệu** |
| Thêm làn 2 (chỉ mục 1-6) | +14 – 17 triệu |

### 4.4 Ghi chú mua hàng

**Camera Dahua:**
- Yêu cầu rõ: **ống kính 3.6mm**, độ phân giải 2MP, hỗ trợ PoE + RTSP
- Test trước khi mua: mở VLC, kéo `rtsp://admin:pass@IP:554/cam/realmonitor?channel=1&subtype=0`

**Đầu đọc thẻ RFID:**
- Ưu tiên loại USB HID (giả lập bàn phím) — không cần driver, cắm là chạy
- Test: mở Notepad, quẹt thẻ → nếu thấy mã hex hiện ra là OK

**Barrier BS-306:**
- Xác nhận với nhà cung cấp: **có nút OPEN trên bo mạch** hoặc cổng kết nối nút rời
- Xác nhận: khi mất điện có **tay quay khẩn cấp** không
- Chọn phiên bản **DC 24V** (nhanh hơn, bền hơn AC)

**Mini PC:**
- Cần ít nhất **2 cổng USB** (1 cho đầu đọc entry, 1 cho exit) — hoặc dùng USB hub
- Cần **LAN port** để kết nối switch PoE
- Nếu có ý định thêm AI sau này → chọn CPU Intel i5 gen 12+

---

## 5. Stack công nghệ

### 5.1 Desktop app (Tauri)

| Layer | Công nghệ | Lý do chọn |
|---|---|---|
| **Framework** | **Tauri 2.x** | Binary nhỏ (~10MB), auto-updater built-in, dùng Vue cho UI |
| **Backend** | **Rust** (stable) | Hiệu năng, an toàn bộ nhớ, single binary |
| **Frontend** | **Vue 3** + Vite | Đã có kinh nghiệm, ecosystem mạnh |
| **UI Library** | **Element Plus** hoặc **Naive UI** | Component có sẵn (table, form, modal) |
| **State** | **Pinia** | Chuẩn của Vue 3 |
| **Router** | **Vue Router** | Điều hướng giữa các màn hình |
| **HTTP client** | **reqwest** (Rust) | Gọi license server |

### 5.2 Rust dependencies

```toml
[dependencies]
tauri = { version = "2", features = ["shell-open"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
serialport = "4"           # Đọc USB RFID (RS232)
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"             # Xử lý datetime
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
image = "0.25"             # Xử lý ảnh
anyhow = "1"               # Error handling
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 5.3 License server (cloud)

| Layer | Công nghệ |
|---|---|
| **Backend** | **FastAPI** (Python 3.11+) |
| **Database** | **PostgreSQL 15** |
| **Auth** | JWT |
| **Frontend** | Vue 3 (admin panel đơn giản) |
| **Hosting** | VPS 2GB RAM (~200k/tháng) hoặc Fly.io/Railway |

> **Lý do Python cho license server:** Đơn giản, phát triển nhanh, không cần hiệu năng cao (chỉ vài request/ngày/khách).

### 5.4 Công cụ phát triển

| Tool | Vai trò |
|---|---|
| **VS Code + rust-analyzer** | IDE |
| **`cargo-watch`** | Auto-reload khi code Rust |
| **`pnpm`** | Package manager cho Vue |
| **DB Browser for SQLite** | Xem DB khi debug |
| **Postman/Insomnia** | Test license server API |
| **`tauri-cli`** | Build, dev, release |

---

## 6. Mô hình dữ liệu

### 6.1 Schema SQLite (local — tại bốt)

```sql
-- ═══════════════════════════════════════════════════════════════
-- BẢNG: cards (thẻ RFID)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE cards (
    id              TEXT PRIMARY KEY,           -- Mã hex từ đầu đọc: "A1B2C3D4"
    card_type       TEXT NOT NULL CHECK(card_type IN ('monthly', 'guest', 'vip')),
    display_code    TEXT NOT NULL,              -- Mã in trên thẻ: "00123"
    owner_name      TEXT,                       -- Tên chủ thẻ (vé tháng)
    owner_phone     TEXT,                       -- SĐT
    vehicle_plate   TEXT,                       -- Biển số đăng ký (vé tháng)
    vehicle_type    TEXT CHECK(vehicle_type IN ('motorbike', 'car', 'truck')),
    valid_from      TIMESTAMP,
    valid_to        TIMESTAMP,                  -- NULL = vĩnh viễn
    status          TEXT DEFAULT 'active' CHECK(status IN ('active', 'lost', 'expired', 'blocked')),
    notes           TEXT,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_cards_status ON cards(status);
CREATE INDEX idx_cards_display_code ON cards(display_code);

-- ═══════════════════════════════════════════════════════════════
-- BẢNG: sessions (phiên đỗ xe)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE sessions (
    id                  TEXT PRIMARY KEY,       -- UUID
    card_id             TEXT NOT NULL REFERENCES cards(id),
    lane                TEXT NOT NULL,          -- 'entry_car', 'entry_bike', 'exit_car', 'exit_bike'
    entry_time          TIMESTAMP NOT NULL,
    entry_photo_path    TEXT,                   -- Đường dẫn tương đối: "photos/2026/11/25/entry_xxx.jpg"
    entry_operator      TEXT,                   -- ID bảo vệ đã xác nhận
    exit_time           TIMESTAMP,
    exit_photo_path     TEXT,
    exit_operator       TEXT,
    duration_minutes    INTEGER,                -- Tính sẵn khi đóng session
    fee                 INTEGER DEFAULT 0,      -- VNĐ
    fee_collected       INTEGER DEFAULT 0,      -- Thực thu
    payment_method      TEXT DEFAULT 'cash' CHECK(payment_method IN ('cash', 'transfer', 'free')),
    status              TEXT DEFAULT 'active' CHECK(status IN ('active', 'completed', 'cancelled', 'lost_card')),
    notes               TEXT,
    created_at          TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_sessions_card_status ON sessions(card_id, status);
CREATE INDEX idx_sessions_status_time ON sessions(status, entry_time DESC);
CREATE INDEX idx_sessions_exit_time ON sessions(exit_time);

-- ═══════════════════════════════════════════════════════════════
-- BẢNG: pricing_rules (bảng giá)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE pricing_rules (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    vehicle_type    TEXT NOT NULL CHECK(vehicle_type IN ('motorbike', 'car', 'truck')),
    minute_from     INTEGER NOT NULL,           -- 0, 120, 480, 1440
    minute_to       INTEGER NOT NULL,           -- 120, 480, 1440, 999999
    price_per_unit  INTEGER NOT NULL,           -- VNĐ cho khung giờ này
    unit_minutes    INTEGER NOT NULL,           -- Đơn vị: 60 = tính theo giờ
    is_active       INTEGER DEFAULT 1,
    effective_from  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ═══════════════════════════════════════════════════════════════
-- BẢNG: operators (bảo vệ + manager)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE operators (
    id              TEXT PRIMARY KEY,           -- UUID
    username        TEXT UNIQUE NOT NULL,
    password_hash   TEXT NOT NULL,              -- bcrypt
    full_name       TEXT NOT NULL,
    role            TEXT NOT NULL CHECK(role IN ('staff', 'manager')),
    phone           TEXT,
    is_active       INTEGER DEFAULT 1,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ═══════════════════════════════════════════════════════════════
-- BẢNG: audit_logs (nhật ký thao tác)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE audit_logs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id      TEXT REFERENCES sessions(id),
    operator_id     TEXT REFERENCES operators(id),
    action          TEXT NOT NULL,              -- 'entry', 'exit', 'override', 'card_lost', 'fee_adjust', 'login'
    metadata        TEXT,                       -- JSON string
    ip_address      TEXT,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_created ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_action ON audit_logs(action);

-- ═══════════════════════════════════════════════════════════════
-- BẢNG: sync_queue (hàng đợi đồng bộ license server)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE sync_queue (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type      TEXT NOT NULL,              -- 'license_check', 'usage_report', 'crash_report'
    payload         TEXT NOT NULL,              -- JSON
    status          TEXT DEFAULT 'pending' CHECK(status IN ('pending', 'synced', 'failed')),
    retry_count     INTEGER DEFAULT 0,
    last_error      TEXT,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    synced_at       TIMESTAMP
);

-- ═══════════════════════════════════════════════════════════════
-- BẢNG: app_config (cấu hình)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE app_config (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
-- Keys: license_key, hardware_id, tenant_name, last_phone_home,
--       grace_period_until, camera_entry_url, camera_exit_url,
--       card_reader_entry_port, card_reader_exit_port
```

### 6.2 Schema PostgreSQL (license server)

```sql
-- ═══════════════════════════════════════════════════════════════
-- BẢNG: customers (khách hàng = tòa nhà)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE customers (
    id              UUID PRIMARY KEY,
    name            VARCHAR(200) NOT NULL,      -- "Chung cư ABC"
    contact_name    VARCHAR(100),
    contact_email   VARCHAR(200),
    contact_phone   VARCHAR(20),
    address         TEXT,
    is_active       BOOLEAN DEFAULT TRUE,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- ═══════════════════════════════════════════════════════════════
-- BẢNG: licenses
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE licenses (
    id                  UUID PRIMARY KEY,
    customer_id         UUID REFERENCES customers(id),
    license_key         VARCHAR(50) UNIQUE NOT NULL,   -- "XXXX-YYYY-ZZZZ-WWWW"
    hardware_id         VARCHAR(128),                  -- SHA256 fingerprint
    max_lanes           INTEGER DEFAULT 2,
    status              VARCHAR(20) DEFAULT 'active' CHECK(status IN ('active', 'suspended', 'revoked', 'expired')),
    valid_from          TIMESTAMPTZ NOT NULL,
    valid_to            TIMESTAMPTZ NOT NULL,
    grace_period_days   INTEGER DEFAULT 30,
    last_phone_home     TIMESTAMPTZ,
    last_app_version    VARCHAR(20),
    notes               TEXT,
    created_at          TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_licenses_key ON licenses(license_key);
CREATE INDEX idx_licenses_customer ON licenses(customer_id);

-- ═══════════════════════════════════════════════════════════════
-- BẢNG: phone_home_logs
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE phone_home_logs (
    id              BIGSERIAL PRIMARY KEY,
    license_id      UUID REFERENCES licenses(id),
    hardware_id     VARCHAR(128),
    app_version     VARCHAR(20),
    ip_address      INET,
    valid           BOOLEAN,
    response_code   INTEGER,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_phone_home_license ON phone_home_logs(license_id, created_at DESC);

-- ═══════════════════════════════════════════════════════════════
-- BẢNG: admin_users (vendor admin)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE admin_users (
    id              UUID PRIMARY KEY,
    username        VARCHAR(100) UNIQUE NOT NULL,
    password_hash   VARCHAR(200) NOT NULL,
    full_name       VARCHAR(200),
    is_active       BOOLEAN DEFAULT TRUE,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- ═══════════════════════════════════════════════════════════════
-- BẢNG: releases (phân phối update)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE releases (
    id              UUID PRIMARY KEY,
    version         VARCHAR(20) NOT NULL,       -- "1.2.3"
    platform        VARCHAR(20) DEFAULT 'windows-x64',
    download_url    TEXT NOT NULL,
    changelog       TEXT,
    is_mandatory    BOOLEAN DEFAULT FALSE,
    released_at     TIMESTAMPTZ DEFAULT NOW()
);
```

### 6.3 Cấu trúc thư mục lưu ảnh

```
%APPDATA%/parking-app/
├── parking.db                # SQLite database
├── photos/
│   └── 2026/
│       └── 11/
│           └── 25/
│               ├── entry_<session_id>.jpg
│               └── exit_<session_id>.jpg
├── logs/
│   └── app-2026-11-25.log
└── backups/
    └── parking-2026-11-25.db
```

---

## 7. Luồng nghiệp vụ

### 7.1 Entry — Xe vào bãi

```
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 1: Xe đến làn vào                                       │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 2: Camera Dahua chụp ảnh                                │
│   - Trigger: bảo vệ bấm "Chụp" HOẶC auto 1 fps               │
│   - Lưu ảnh tạm: photos/temp/entry_<uuid>.jpg                │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 3: Khách/bảo vệ quẹt thẻ RFID                           │
│   - Đầu đọc gửi mã hex → Rust backend                        │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 4: Backend xử lý                                        │
│   ├─ Kiểm tra thẻ có tồn tại? (query cards table)            │
│   │   └─ Không có → CẢNH BÁO "Thẻ không hợp lệ"              │
│   ├─ Kiểm tra status = 'active'?                             │
│   │   └─ Không → CẢNH BÁO "Thẻ bị khóa/mất"                  │
│   ├─ Nếu card_type = 'monthly':                              │
│   │   ├─ valid_to > now()? → OK                              │
│   │   └─ Hết hạn → CẢNH BÁO "Vé tháng hết hạn"               │
│   ├─ Nếu card_type = 'guest': OK (cho vào)                   │
│   └─ Kiểm tra thẻ có đang có session active?                 │
│       └─ Có → CẢNH BÁO "Thẻ đang trong bãi" (chưa ra)        │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 5: Nếu OK → Tạo session                                 │
│   INSERT INTO sessions (                                     │
│     id=UUID, card_id, lane, entry_time=now(),                │
│     entry_photo_path, entry_operator, status='active'        │
│   )                                                          │
│   - Di chuyển ảnh từ temp/ → photos/YYYY/MM/DD/              │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 6: (Tùy chọn) In vé cho khách                           │
│   - Vé giấy 80mm, gồm:                                       │
│     * Mã vé (session_id 8 ký tự đầu)                         │
│     * Mã thẻ (display_code)                                  │
│     * Giờ vào                                                 │
│     * Loại vé                                                 │
│     * QR code (link đến session — tùy chọn)                  │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 7: Ghi audit log                                        │
│   INSERT INTO audit_logs (session_id, operator_id, action,   │
│     metadata={card_id, lane, fee_policy='none'})             │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 8: 👤 BẢO VỆ bấm NÚT VẬT LÝ mở barrier                  │
│   - Phần mềm KHÔNG điều khiển barrier                        │
│   - Barrier do bảo vệ điều khiển trực tiếp                   │
└──────────────────────────────────────────────────────────────┘
```

### 7.2 Exit — Xe ra bãi

```
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 1: Xe đến làn ra                                        │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 2: Khách quẹt thẻ RFID                                  │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 3: Backend tìm session đang active                      │
│   SELECT * FROM sessions WHERE card_id=? AND status='active' │
│   ├─ Không có → CẢNH BÁO "Không tìm thấy phiên đỗ xe"        │
│   │   → Chuyển sang luồng "Mất thẻ" (xem 7.4)                │
│   └─ Có → tiếp tục                                            │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 4: Chụp ảnh xe lúc ra                                   │
│   - Lưu tạm: photos/temp/exit_<session_id>.jpg               │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 5: Tính phí                                             │
│   duration_min = (now() - entry_time) / 60                   │
│   ├─ Nếu card_type = 'monthly' và còn hạn → fee = 0          │
│   ├─ Nếu card_type = 'vip' → fee = 0                         │
│   └─ Nếu card_type = 'guest' hoặc 'monthly' hết hạn:         │
│       fee = sum theo bảng pricing_rules                      │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 6: Hiển thị cho bảo vệ                                  │
│   ┌────────────────────────────────────────┐                 │
│   │ ẢNH VÀO    ⏱️ 08:15 → 17:30: 9h15'     │                 │
│   │ [ảnh]      💰 Phí: 45.000đ             │                 │
│   │                                        │                 │
│   │ ẢNH RA     🎫 Thẻ: 00123 (khách)       │                 │
│   │ [ảnh]                                  │                 │
│   │                                        │                 │
│   │ ⚠️ So 2 ảnh — cùng 1 xe?               │                 │
│   │                                        │                 │
│   │ [✅ ĐÃ THU TIỀN] [❌ TỪ CHỐI]          │                 │
│   └────────────────────────────────────────┘                 │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 7: Bảo vệ xác nhận                                      │
│   - Nhìn 2 ảnh → so với xe thực tế                           │
│   - Thu tiền mặt (nếu có phí)                                │
│   - Bấm "ĐÃ THU TIỀN"                                        │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 8: Cập nhật session                                     │
│   UPDATE sessions SET                                        │
│     exit_time=now(),                                         │
│     exit_photo_path,                                         │
│     exit_operator,                                           │
│     duration_minutes=?, fee=?, fee_collected=?,              │
│     status='completed'                                       │
│   WHERE id=?                                                 │
│   - Di chuyển ảnh từ temp/ → photos/YYYY/MM/DD/              │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 9: In hóa đơn (nếu cần)                                 │
│   Ghi audit log exit                                         │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│ BƯỚC 10: 👤 BẢO VỆ bấm NÚT VẬT LÝ mở barrier                 │
└──────────────────────────────────────────────────────────────┘
```

### 7.3 Tính phí — Chi tiết

**Ví dụ bảng giá xe máy:**

| minute_from | minute_to | price | Ý nghĩa |
|---|---|---|---|
| 0 | 120 | 5.000 | 2 giờ đầu: 5k |
| 120 | 1440 | 3.000 | Mỗi giờ tiếp theo (đến 24h): 3k/giờ |
| 1440 | 999999 | 50.000 | Qua ngày: 50k/ngày |

**Ví dụ bảng giá ô tô:**

| minute_from | minute_to | price | Ý nghĩa |
|---|---|---|---|
| 0 | 60 | 15.000 | 1 giờ đầu: 15k |
| 60 | 1440 | 10.000 | Mỗi giờ tiếp: 10k/giờ |
| 1440 | 999999 | 200.000 | Qua ngày: 200k/ngày |

**Công thức tính:**

```
if card_type in ('monthly' and valid) or card_type == 'vip':
    return 0

total = 0
for rule in pricing_rules where vehicle_type = card.vehicle_type and is_active = 1:
    overlap = min(duration_minutes, rule.minute_to) - rule.minute_from
    if overlap > 0:
        units = ceil(overlap / rule.unit_minutes)
        total += units * rule.price_per_unit

return total
```

**Test case:**
- Xe máy đỗ 3 giờ = 180 phút
  - 0-120: 2 giờ đầu = 5.000
  - 120-180: 1 giờ tiếp = 3.000
  - **Tổng: 8.000**

### 7.4 Xử lý mất thẻ

```
BƯỚC 1: Bảo vệ mở màn hình "Tra cứu mất thẻ"
BƯỚC 2: Nhập khoảng giờ vào ước tính (VD: 7h30 - 8h30)
BƯỚC 3: Hệ thống query:
   SELECT * FROM sessions 
   WHERE status='active' 
     AND entry_time BETWEEN '7:30' AND '8:30'
BƯỚC 4: Hiện danh sách session (mỗi item có ảnh entry + giờ)
BƯỚC 5: Bảo vệ scroll ảnh, tìm ảnh khớp xe đang ra
BƯỚC 6: Chọn session → tính phí → thu tiền
BƯỚC 7: Đánh dấu thẻ cũ:
   UPDATE cards SET status='lost' WHERE id=old_card_id
BƯỚC 8: (Tùy chọn) Phát thẻ mới cho khách
BƯỚC 9: Ghi audit:
   action='card_lost', metadata={old_card_id, new_session_id, operator_id}
```

### 7.5 Các trường hợp đặc biệt

| Tình huống | Xử lý |
|---|---|
| **Xe vào rồi ra trong 1 phút** | Vẫn tính phí (min 1 unit) |
| **Thẻ đang active nhưng khách muốn vào lần 2** | Cảnh báo "Thẻ đang trong bãi" — có thể do khách chưa quẹt ra |
| **Mất điện giữa phiên** | Session vẫn active trong DB. Khi có điện, bảo vệ nhập thông tin thủ công |
| **Đầu đọc thẻ hỏng** | Nhập mã thẻ thủ công qua bàn phím (mã in trên thẻ) |
| **Camera hỏng** | Bảo vệ dùng điện thoại chụp, upload thủ công qua app |
| **Khách quỵt tiền** | Bảo vệ từ chối mở barrier. Ghi audit `refuse_exit` |
| **Xe ra không quẹt thẻ** | Bảo vệ nhập thủ công mã thẻ nếu biết, hoặc coi như mất thẻ |

---

## 8. Thiết kế UI

### 8.1 Danh sách màn hình

| # | Màn hình | Vai trò | Chức năng |
|---|---|---|---|
| S1 | Login | Tất cả | Đăng nhập |
| S2 | Entry - Lane | Bảo vệ | Xử lý xe vào |
| S3 | Exit - Lane | Bảo vệ | Xử lý xe ra |
| S4 | Lost Card | Bảo vệ + Manager | Tra cứu mất thẻ |
| S5 | Shift Report | Bảo vệ | Xem doanh thu ca mình |
| S6 | Dashboard | Manager | Báo cáo tổng quan |
| S7 | Card Management | Manager | CRUD thẻ |
| S8 | Pricing Config | Manager | Cấu hình bảng giá |
| S9 | Operator Management | Manager | Quản lý bảo vệ |
| S10 | Audit Log | Manager | Xem log thao tác |
| S11 | Settings | Manager | Cấu hình camera, đầu đọc |

### 8.2 Mockup S2 — Entry Lane

```
┌──────────────────────────────────────────────────────────────┐
│  BỐT VÀO — LÀN Ô TÔ      🟢 Camera   🟢 Đầu đọc   🟢 License│
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────────────────────────┐  ┌──────────────┐ │
│  │                                     │  │              │ │
│  │       ẢNH CHỤP XE VỪA VÀO           │  │ Live camera  │ │
│  │                                     │  │ (preview)    │ │
│  │                                     │  │              │ │
│  └─────────────────────────────────────┘  └──────────────┘ │
│                                                              │
│  ⏱️ Giờ vào: 08:15:32                                       │
│                                                              │
│  🎫 Thẻ quẹt: 00123 (VÉ THÁNG — còn hạn 45 ngày)            │
│      Chủ thẻ: Nguyễn Văn A — 0901 234 567                   │
│      Biển số: 51A-123.45                                     │
│                                                              │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐    │
│  │ 📸 Chụp lại  │ │ 🖨️ In vé     │ │ ✅ XÁC NHẬN VÀO  │    │
│  │   [F5]       │ │   [F6]       │ │   (Enter)        │    │
│  └──────────────┘ └──────────────┘ └──────────────────┘    │
│                                                              │
│  ═══════════════════════════════════════════════════════    │
│  💡 SAU KHI XÁC NHẬN: Bấm nút vật lý để mở barrier          │
└──────────────────────────────────────────────────────────────┘
```

### 8.3 Mockup S3 — Exit Lane

```
┌──────────────────────────────────────────────────────────────┐
│  BỐT RA — LÀN Ô TÔ       🟢 Camera   🟢 Đầu đọc   🟢 License│
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌───────────────────┐  ┌───────────────────┐              │
│  │                   │  │                   │              │
│  │   ẢNH LÚC VÀO     │  │   ẢNH LÚC RA      │              │
│  │   08:15:32        │  │   17:30:15        │              │
│  │                   │  │                   │              │
│  └───────────────────┘  └───────────────────┘              │
│                                                              │
│  ⚠️ SO 2 ẢNH — CÓ PHẢI CÙNG 1 XE KHÔNG?                     │
│                                                              │
│  🎫 Thẻ: 00123 (KHÁCH VÃNG LAI)                             │
│  ⏱️ Thời gian đỗ: 9 giờ 15 phút                             │
│  💰 Phí: 45.000đ                                             │
│                                                              │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐    │
│  │ ❌ TỪ CHỐI   │ │ 🔍 MẤT THẺ   │ │ ✅ ĐÃ THU TIỀN   │    │
│  │   [Esc]      │ │   [F7]       │ │   (Enter)        │    │
│  └──────────────┘ └──────────────┘ └──────────────────┘    │
│                                                              │
│  💡 SAU KHI XÁC NHẬN: Bấm nút vật lý để mở barrier          │
└──────────────────────────────────────────────────────────────┘
```

### 8.4 Keyboard shortcuts (bắt buộc)

| Phím | Chức năng | Màn hình |
|---|---|---|
| `Enter` | Xác nhận (vào/ra) | S2, S3 |
| `Esc` | Từ chối | S2, S3 |
| `F2` | Sửa mã thẻ | S2, S3 |
| `F5` | Chụp lại ảnh | S2, S3 |
| `F6` | In vé | S2 |
| `F7` | Mở màn hình mất thẻ | S3 |
| `F9` | In hóa đơn | S3 |
| `F12` | Xem audit log nhanh | Tất cả |

### 8.5 Design system

| Yếu tố | Giá trị |
|---|---|
| **Màu chính** | `#1E88E5` (blue) |
| **Màu thành công** | `#43A047` (green) |
| **Màu cảnh báo** | `#FB8C00` (orange) |
| **Màu nguy hiểm** | `#E53935` (red) |
| **Font** | Inter hoặc Roboto, size 14-16px |
| **Kích thước nút chính** | ≥ 48×48px (dễ bấm cảm ứng) |
| **Độ tương phản** | WCAG AA (4.5:1) |
| **Ngôn ngữ** | Tiếng Việt (mặc định) |

---

## 9. Cấu trúc mã nguồn

### 9.1 Cấu trúc dự án

```
parking-app/
│
├── src-tauri/                        # ═══ RUST BACKEND ═══
│   ├── Cargo.toml
│   ├── tauri.conf.json               # Cấu hình Tauri
│   ├── icons/
│   └── src/
│       ├── main.rs                   # Entry point
│       ├── lib.rs
│       │
│       ├── commands/                 # Tauri IPC commands
│       │   ├── mod.rs
│       │   ├── entry.rs              # create_session, capture_entry
│       │   ├── exit.rs               # find_session, calc_fee, complete_session
│       │   ├── cards.rs              # CRUD thẻ
│       │   ├── auth.rs               # login/logout
│       │   ├── reports.rs            # báo cáo
│       │   └── settings.rs           # cấu hình
│       │
│       ├── hardware/                 # ═══ HAL ═══
│       │   ├── mod.rs
│       │   ├── interfaces/
│       │   │   ├── mod.rs
│       │   │   ├── card_reader.rs    # trait ICardReader
│       │   │   └── camera.rs         # trait ICamera
│       │   ├── adapters/
│       │   │   ├── mod.rs
│       │   │   ├── rfid_usb.rs       # RFIDAdapter
│       │   │   └── dahua_rtsp.rs     # DahuaAdapter
│       │   └── factory.rs
│       │
│       ├── db/                       # ═══ DATABASE ═══
│       │   ├── mod.rs
│       │   ├── connection.rs
│       │   ├── migrations.rs
│       │   ├── models/
│       │   │   ├── card.rs
│       │   │   ├── session.rs
│       │   │   ├── pricing.rs
│       │   │   └── operator.rs
│       │   └── repositories/
│       │       ├── card_repo.rs
│       │       ├── session_repo.rs
│       │       └── pricing_repo.rs
│       │
│       ├── core/                     # ═══ BUSINESS LOGIC ═══
│       │   ├── mod.rs
│       │   ├── fee_calculator.rs     # Tính phí
│       │   ├── session_service.rs    # Tạo/đóng session
│       │   ├── auth_service.rs
│       │   └── audit_service.rs
│       │
│       ├── license/                  # ═══ LICENSE ═══
│       │   ├── mod.rs
│       │   ├── phone_home.rs         # Gọi license server
│       │   ├── hardware_fingerprint.rs
│       │   └── grace_period.rs
│       │
│       └── utils/
│           ├── mod.rs
│           ├── image.rs              # Lưu ảnh, resize
│           ├── printer.rs            # In nhiệt ESC/POS
│           └── logger.rs
│
├── src/                              # ═══ VUE FRONTEND ═══
│   ├── main.js
│   ├── App.vue
│   ├── router/
│   │   └── index.js
│   ├── stores/
│   │   ├── auth.js
│   │   ├── session.js
│   │   └── settings.js
│   ├── api/                          # Wrapper cho Tauri invoke
│   │   ├── tauri.js
│   │   ├── entry.js
│   │   ├── exit.js
│   │   └── ...
│   ├── views/
│   │   ├── Login.vue
│   │   ├── EntryLane.vue
│   │   ├── ExitLane.vue
│   │   ├── LostCard.vue
│   │   ├── ShiftReport.vue
│   │   ├── manager/
│   │   │   ├── Dashboard.vue
│   │   │   ├── Cards.vue
│   │   │   ├── Pricing.vue
│   │   │   ├── Operators.vue
│   │   │   ├── AuditLog.vue
│   │   │   └── Settings.vue
│   │   └── common/
│   │       ├── Forbidden.vue
│   │       └── NotFound.vue
│   ├── components/
│   │   ├── PhotoCard.vue
│   │   ├── CardInfo.vue
│   │   ├── FeeDisplay.vue
│   │   ├── ConnectionBadge.vue
│   │   └── ...
│   └── assets/
│
├── package.json
├── vite.config.js
├── pnpm-lock.yaml
└── README.md
```

### 9.2 Ví dụ code Rust — Entry command

```rust
// src-tauri/src/commands/entry.rs
use crate::db::repositories::{card_repo, session_repo};
use crate::core::audit_service;
use crate::hardware::factory;
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateEntryInput {
    pub card_id: String,
    pub lane: String,
    pub operator_id: String,
}

#[derive(Serialize)]
pub struct CreateEntryResult {
    pub session_id: String,
    pub card_display_code: String,
    pub card_type: String,
    pub owner_name: Option<String>,
    pub vehicle_plate: Option<String>,
    pub entry_time: String,
    pub photo_path: String,
}

#[tauri::command]
pub async fn create_entry(
    input: CreateEntryInput,
    db: State<'_, DbPool>,
) -> Result<CreateEntryResult, String> {
    // 1. Kiểm tra thẻ
    let card = card_repo::find_by_id(&db, &input.card_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Thẻ không hợp lệ".to_string())?;
    
    if card.status != "active" {
        return Err(format!("Thẻ đang ở trạng thái: {}", card.status));
    }
    
    // 2. Kiểm tra vé tháng hết hạn
    if card.card_type == "monthly" {
        let now = chrono::Utc::now().naive_utc();
        if let Some(valid_to) = card.valid_to {
            if valid_to < now {
                return Err("Vé tháng đã hết hạn".to_string());
            }
        }
    }
    
    // 3. Kiểm tra thẻ đang có session active
    if session_repo::find_active_by_card(&db, &input.card_id)?.is_some() {
        return Err("Thẻ đang trong bãi (chưa quẹt ra)".to_string());
    }
    
    // 4. Chụp ảnh
    let camera = factory::get_camera_for_lane(&input.lane)?;
    let photo_path = camera.capture_snapshot()?;
    
    // 5. Tạo session
    let session_id = uuid::Uuid::new_v4().to_string();
    let entry_time = chrono::Utc::now().naive_utc();
    
    session_repo::create(&db, &session_id, &input.card_id, &input.lane,
                          entry_time, &photo_path, &input.operator_id)?;
    
    // 6. Audit log
    audit_service::log(&db, Some(&session_id), Some(&input.operator_id),
                       "entry", Some(format!("card_id={}", input.card_id)))?;
    
    Ok(CreateEntryResult {
        session_id,
        card_display_code: card.display_code,
        card_type: card.card_type,
        owner_name: card.owner_name,
        vehicle_plate: card.vehicle_plate,
        entry_time: entry_time.format("%H:%M:%S").to_string(),
        photo_path,
    })
}
```

### 9.3 Ví dụ code Vue — EntryLane.vue

```vue
<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

const currentSession = ref(null)
const cardInfo = ref(null)
const loading = ref(false)
const errorMsg = ref('')

// Lắng nghe sự kiện quẹt thẻ từ Rust
let unlisten
onMounted(async () => {
  unlisten = await listen('card-tapped', async (event) => {
    const cardId = event.payload.card_id
    await handleCardTap(cardId)
  })
})
onUnmounted(() => unlisten?.())

async function handleCardTap(cardId) {
  loading.value = true
  errorMsg.value = ''
  try {
    const result = await invoke('create_entry', {
      input: { card_id: cardId, lane: 'entry_car', operator_id: 'current_user' }
    })
    currentSession.value = result
    cardInfo.value = result
  } catch (e) {
    errorMsg.value = e.toString()
  } finally {
    loading.value = false
  }
}

async function confirmEntry() {
  await invoke('confirm_entry', { sessionId: currentSession.value.session_id })
  currentSession.value = null
  cardInfo.value = null
}

async function captureAgain() {
  if (!currentSession.value) return
  const newPhoto = await invoke('capture_entry_again', {
    sessionId: currentSession.value.session_id
  })
  currentSession.value.photo_path = newPhoto
}

function reject() {
  currentSession.value = null
  cardInfo.value = null
  errorMsg.value = ''
}

// Keyboard shortcuts
function onKeydown(e) {
  if (e.key === 'Enter' && currentSession.value) confirmEntry()
  if (e.key === 'Escape') reject()
  if (e.key === 'F5') captureAgain()
}
onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="entry-lane">
    <header>
      <h1>BỐT VÀO — LÀN Ô TÔ</h1>
      <ConnectionBadges />
    </header>

    <div v-if="errorMsg" class="alert alert-error">
      ⚠️ {{ errorMsg }}
    </div>

    <div v-if="currentSession" class="content">
      <div class="photo-panel">
        <img :src="`asset://${currentSession.photo_path}`" alt="Entry photo" />
        <p class="timestamp">⏱️ {{ currentSession.entry_time }}</p>
      </div>

      <div class="info-panel">
        <CardInfo
          :card-code="currentSession.card_display_code"
          :card-type="currentSession.card_type"
          :owner="currentSession.owner_name"
          :plate="currentSession.vehicle_plate"
        />
        
        <div class="actions">
          <button @click="captureAgain" class="btn-secondary">📸 Chụp lại (F5)</button>
          <button @click="confirmEntry" class="btn-primary">✅ XÁC NHẬN (Enter)</button>
        </div>
        
        <p class="hint">💡 SAU KHI XÁC NHẬN: Bấm nút vật lý để mở barrier</p>
      </div>
    </div>

    <div v-else class="waiting">
      <p>Đang chờ quẹt thẻ...</p>
    </div>
  </div>
</template>
```

---

## 10. License management

### 10.1 Luồng cấp license

```
VENDOR                        LICENSE SERVER                CUSTOMER (app)
  │                                  │                            │
  │ 1. Tạo khách hàng mới            │                            │
  ├─────────────────────────────────→│                            │
  │                                  │                            │
  │ 2. Tạo license key               │                            │
  ├─────────────────────────────────→│                            │
  │                                  │                            │
  │ 3. Gửi file .exe + license key   │                            │
  ├──────────────────────────────────┼───────────────────────────→│
  │                                  │                            │
  │                                  │ 4. App khởi động lần đầu   │
  │                                  │←───────────────────────────┤
  │                                  │    POST /api/activate       │
  │                                  │    {license_key, hw_id}     │
  │                                  │                            │
  │                                  │ 5. Server ghi hw_id        │
  │                                  │    Trả về OK + expires_at   │
  │                                  ├───────────────────────────→│
  │                                  │                            │
  │                                  │ 6. Mỗi 24h phone-home      │
  │                                  │←───────────────────────────┤
  │                                  │    POST /api/verify         │
  │                                  │                            │
  │                                  │ 7. Server check status      │
  │                                  │    Trả về valid/expired     │
  │                                  ├───────────────────────────→│
```

### 10.2 API endpoints (license server)

| Method | Endpoint | Mô tả |
|---|---|---|
| `POST` | `/api/activate` | Đăng ký hardware_id cho license lần đầu |
| `POST` | `/api/verify` | Phone-home định kỳ (24h) |
| `POST` | `/api/change-hardware` | Đổi máy (cần admin duyệt) |
| `GET` | `/api/releases/latest` | Check update mới |
| `POST` | `/api/usage` | Báo cáo usage (số session, số xe, etc.) |
| `POST` | `/api/crash-report` | Báo cáo lỗi (opt-in) |

### 10.3 Payload mẫu

**Activate request:**
```json
POST /api/activate
{
  "license_key": "XXXX-YYYY-ZZZZ-WWWW",
  "hardware_id": "sha256:a1b2c3d4e5f6...",
  "app_version": "1.0.0",
  "os": "Windows 11"
}
```

**Response:**
```json
{
  "valid": true,
  "customer_name": "Chung cư ABC",
  "expires_at": "2027-11-25T00:00:00Z",
  "grace_period_days": 30,
  "max_lanes": 2,
  "features": ["basic", "reports", "printer"]
}
```

**Verify request (mỗi 24h):**
```json
POST /api/verify
{
  "license_key": "XXXX-YYYY-ZZZZ-WWWW",
  "hardware_id": "sha256:a1b2c3d4e5f6...",
  "app_version": "1.0.0"
}
```

**Response khi bị khóa:**
```json
{
  "valid": false,
  "reason": "suspended",
  "message": "License đã bị tạm khóa do chưa thanh toán",
  "grace_period_until": "2026-12-25T00:00:00Z"
}
```

### 10.4 Grace period — Bảng trạng thái

| Trạng thái | Hành vi app |
|---|---|
| License hợp lệ, check-in < 24h | Chạy bình thường |
| License hợp lệ, check-in < 7 ngày | Chạy bình thường (offline OK) |
| License hợp lệ, check-in 7-30 ngày | Chạy + banner vàng "Sắp mất kết nối, vui lòng kết nối Internet" |
| License hợp lệ, check-in 30-60 ngày | Chạy + banner đỏ "Mất kết nối lâu, sắp bị khóa" |
| License hợp lệ, check-in > 60 ngày | **Khóa app**, chỉ cho export dữ liệu + in báo cáo |
| License bị suspend (server) | Chạy trong grace_period_until |
| License hết hạn | Khóa ngay, hiển thị thông báo gia hạn |

### 10.5 Hardware fingerprint

```rust
// src-tauri/src/license/hardware_fingerprint.rs
pub fn get_hardware_id() -> String {
    let mut hasher = Sha256::new();
    
    // MAC address của card mạng chính
    if let Ok(mac) = mac_address::get_mac_address() {
        hasher.update(mac.to_string().as_bytes());
    }
    
    // Serial ổ cứng (Windows: WMI, Linux: /sys/block/*/serial)
    #[cfg(target_os = "windows")]
    {
        if let Ok(serial) = get_disk_serial() {
            hasher.update(serial.as_bytes());
        }
    }
    
    // CPU ID
    if let Ok(cpu_id) = get_cpu_id() {
        hasher.update(cpu_id.as_bytes());
    }
    
    format!("sha256:{}", hex::encode(hasher.finalize()))
}
```

---

## 11. Xử lý sự cố & offline

### 11.1 Ma trận sự cố

| Sự cố | Phần mềm | Barrier | Fallback |
|---|---|---|---|
| **Mất Internet (LAN OK)** | ✅ Chạy đủ (local DB) | ✅ OK | Không cần |
| **Mất LAN (switch chết)** | ⚠️ RFID USB OK, camera mất | ✅ OK | Chụp ảnh bằng điện thoại, nhập mã thẻ tay |
| **Mất điện** | ❌ Tắt (UPS ~20 phút) | ✅ Tay quay | Sổ giấy + tay quay |
| **Mini PC hỏng** | ❌ Chết | ✅ OK | Sổ giấy + nút bấm barrier |
| **Camera hỏng** | ⚠️ Không chụp được | ✅ OK | Dùng điện thoại chụp |
| **Đầu đọc thẻ hỏng** | ⚠️ Không quẹt được | ✅ OK | Nhập mã thẻ tay (in trên thẻ) |
| **Barrier hỏng** | ✅ Chạy đủ | ❌ Không mở được | Gọi thợ, mở tay |

### 11.2 Chế độ offline

**Khi app khởi động mà không có mạng:**

1. App check `last_phone_home` trong DB
2. Nếu < 60 ngày → chạy bình thường + banner vàng
3. Nếu 30-60 ngày → chạy + banner đỏ
4. Nếu > 60 ngày → khóa, chỉ cho export

**Khi có mạng lại:**
- Auto phone-home trong vòng 60 giây
- Đẩy `sync_queue` (nếu có)
- Xóa banner

### 11.3 Sync queue (nếu cần báo cáo lên cloud)

Với v1.0, **không có sync queue** — vì DB local là source of truth. Chỉ sync **license check-in** và **crash report** (opt-in).

Nếu sau này muốn có dashboard tập trung cho vendor xem tất cả khách → thêm module sync (không bắt buộc).

### 11.4 Backup tự động

```rust
// Chạy hàng ngày lúc 02:00
fn daily_backup() -> Result<()> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d");
    let backup_path = format!("backups/parking-{}.db", timestamp);
    std::fs::copy("parking.db", backup_path)?;
    
    // Giữ 30 bản gần nhất
    cleanup_old_backups(30)?;
    Ok(())
}
```

**Backup manual:** Menu "Settings → Backup ngay" → chọn folder lưu (USB, external drive).

---

## 12. Bảo mật & phân quyền

### 12.1 Phân quyền

| Chức năng | Staff | Manager |
|---|---|---|
| Đăng nhập | ✅ | ✅ |
| Xem màn hình entry/exit | ✅ | ✅ |
| Tạo session | ✅ | ✅ |
| Đóng session | ✅ | ✅ |
| Xử lý mất thẻ | ✅ | ✅ |
| Xem ca trực của mình | ✅ | ✅ |
| Xem báo cáo toàn hệ thống | ❌ | ✅ |
| Quản lý thẻ | ❌ | ✅ |
| Cấu hình bảng giá | ❌ | ✅ |
| Quản lý nhân viên | ❌ | ✅ |
| Xem audit log | ❌ | ✅ |
| Cấu hình camera, đầu đọc | ❌ | ✅ |
| Export dữ liệu | ❌ | ✅ |
| Backup | ❌ | ✅ |

### 12.2 Authentication

- Password hash: **bcrypt** (cost 12)
- Session timeout: **8 giờ** (một ca trực)
- Auto-lock sau **5 phút** không thao tác
- Mở khóa bằng password (không cần logout)

### 12.3 Audit log — Bắt buộc ghi

Mọi hành động sau **phải** ghi audit:

- Login, logout, failed login
- Create/complete/cancel session
- Override (mở barrier thủ công cho khách quỵt, mất thẻ, etc.)
- Sửa mã thẻ thủ công
- Điều chỉnh phí
- CRUD thẻ, bảng giá, nhân viên
- Backup, export dữ liệu
- Thay đổi cấu hình

### 12.4 Data integrity

- Foreign key constraints: **BẬT** (`PRAGMA foreign_keys = ON`)
- Transactions cho mọi thao tác multi-step
- Soft delete cho cards (không xóa cứng nếu đã có session)
- Audit log **không bao giờ xóa** — archive sau 5 năm

---

## 13. Lộ trình phát triển

### 13.1 Milestones

#### **Milestone 1: PoC — Chứng minh kỹ thuật** (2-3 tuần)

**Mục tiêu:** Chứng minh các mảnh ghép hardware + software hoạt động được với nhau.

**Deliverables:**
- [ ] Tauri app skeleton chạy được trên mini PC
- [ ] Đọc mã thẻ RFID qua USB, hiển thị lên UI
- [ ] Chụp ảnh từ camera Dahua qua RTSP, lưu file
- [ ] SQLite kết nối OK, tạo bảng cơ bản
- [ ] Mockup UI 2 màn hình (entry/exit)
- [ ] **Test thực tế:** quẹt thẻ → hiện ảnh → lưu DB → hiển thị

**Rủi ro cần kiểm chứng:**
- Đầu đọc thẻ USB HID có hoạt động ổn định không
- Camera Dahua có ổn định trong lấy ảnh không (độ trễ, chất lượng ảnh)
- Tauri build ra .exe có chạy được không

#### **Milestone 2: MVP — Luồng cơ bản end-to-end** (4-6 tuần)

**Mục tiêu:** Bảo vệ có thể dùng phần mềm thay sổ giấy.

**Deliverables:**
- [ ] Database schema đầy đủ (6 bảng)
- [ ] Màn hình login + phân quyền
- [ ] Màn hình entry: quẹt thẻ → hiện ảnh → xác nhận → tạo session
- [ ] Màn hình exit: quẹt thẻ → hiện 2 ảnh → xác nhận → đóng session
- [ ] Tính phí theo bảng giá hard-code
- [ ] Audit log cơ bản
- [ ] **Test thực tế:** 2 BS-306 + 2 camera + 2 đầu đọc → 1 tuần vận hành thử

**Definition of done:**
- Bảo vệ chạy được 1 ca trực hoàn chỉnh mà không cần support
- Không có data loss khi tắt/mở app

#### **Milestone 3: Hoàn thiện nghiệp vụ** (4-6 tuần)

**Deliverables:**
- [ ] CRUD thẻ (thêm, sửa, xóa, khóa)
- [ ] CRUD bảng giá
- [ ] Quản lý nhân viên
- [ ] Màn hình mất thẻ
- [ ] In vé + in hóa đơn (máy in nhiệt)
- [ ] Báo cáo doanh thu (ngày, ca, tháng)
- [ ] Export Excel/PDF
- [ ] Audit log đầy đủ + màn hình xem
- [ ] Backup tự động

#### **Milestone 4: License + Distribution** (2-3 tuần)

**Deliverables:**
- [ ] License server (FastAPI + PostgreSQL)
- [ ] Admin panel cho vendor
- [ ] Phone-home trong app (24h)
- [ ] Grace period logic
- [ ] Hardware fingerprint
- [ ] Auto-updater (Tauri built-in)
- [ ] Build installer .exe + .msi

#### **Milestone 5: Ổn định & bàn giao** (2-3 tuần)

**Deliverables:**
- [ ] Test tất cả kịch bản sự cố (mất mạng, mất điện, mất thẻ)
- [ ] Performance test (100 xe/ngày)
- [ ] Tài liệu hướng dẫn sử dụng (cho bảo vệ)
- [ ] Tài liệu kỹ thuật (cho bạn maintain)
- [ ] Video hướng dẫn cài đặt
- [ ] Pilot 1 tòa nhà 2 tuần → fix bug → release v1.0

### 13.2 Timeline tổng

| Milestone | Thời gian | Tích lũy |
|---|---|---|
| M1: PoC | 2-3 tuần | 3 tuần |
| M2: MVP | 4-6 tuần | 9 tuần |
| M3: Hoàn thiện | 4-6 tuần | 15 tuần |
| M4: License | 2-3 tuần | 18 tuần |
| M5: Ổn định | 2-3 tuần | 21 tuần |
| **Tổng** | | **~5 tháng** |

---

## 14. Deployment & vận hành

### 14.1 Build & đóng gói

```bash
# Build production
cd parking-app
pnpm install
pnpm tauri build

# Output:
# src-tauri/target/release/bundle/
# ├── msi/parking-app_1.0.0_x64_en-US.msi
# └── nsis/parking-app_1.0.0_x64-setup.exe
```

### 14.2 Quy trình cài đặt cho khách hàng

**Trước khi cài:**
- Mini PC chạy Windows 10/11 (64-bit)
- Kết nối mạng LAN với camera và switch PoE
- Đầu đọc thẻ cắm USB

**Cài đặt (15 phút):**

1. Chạy file `.exe` installer
2. Next → Next → Install (mặc định ổ C)
3. Mở app lần đầu → nhập **license key** (vendor cung cấp)
4. Cấu hình ban đầu:
   - IP camera entry + exit
   - Port đầu đọc thẻ (tự detect)
   - Tên tòa nhà
5. Tạo tài khoản manager đầu tiên
6. Tạo tài khoản bảo vệ
7. Cấu hình bảng giá
8. Nhập thẻ vé tháng (nếu có sẵn khách)

**Checklist sau cài:**
- [ ] Camera hiện ảnh OK
- [ ] Quẹt thẻ hiện mã OK
- [ ] In thử 1 vé OK
- [ ] Tạo thử 1 session OK
- [ ] Phone-home license OK

### 14.3 Monitoring cho vendor

**License server dashboard hiển thị:**

| Chỉ số | Ý nghĩa |
|---|---|
| Danh sách customers | Ai đang dùng |
| License status | Active, expired, suspended |
| Last phone-home | Ai không check-in > 7 ngày |
| App version | Ai chưa update |
| Usage metrics | Số session, số xe (opt-in) |
| Crash reports | Lỗi phổ biến (opt-in) |

**Alert tự động:**
- License sắp hết hạn (< 30 ngày) → email trước 30/7/1 ngày
- Khách không phone-home > 14 ngày → email hỏi thăm
- Crash rate > 5% → email cảnh báo

### 14.4 Chi phí vận hành (vendor)

| Hạng mục | Chi phí/tháng |
|---|---|
| VPS license server (2GB RAM) | 200.000đ |
| Domain + SSL | 50.000đ |
| Backup cloud | 50.000đ |
| **Tổng** | **~300.000đ/tháng** |

Cho dù có 1 khách hay 50 khách → chi phí gần như nhau (VPS nhỏ là đủ).

---

## 15. Rủi ro & giảm thiểu

### 15.1 Rủi ro kỹ thuật

| Rủi ro | Khả năng | Tác động | Giảm thiểu |
|---|---|---|---|
| Đầu đọc thẻ USB không ổn định | Trung bình | Cao | Test kỹ M1. Có fallback nhập tay |
| Camera Dahua RTSP chậm/giật | Thấp | Trung bình | Test trước khi mua. Có fallback chụp bằng đt |
| Tauri build lỗi trên Windows | Trung bình | Trung bình | Test build sớm M1. Có CI/CD |
| SQLite corrupt khi mất điện | Thấp | Cao | Bật WAL mode + UPS + backup daily |
| License bị bypass | Trung bình | Thấp | Code obfuscation + phone-home bắt buộc |
| Rust học khó, chậm | Cao | Cao | Nếu chưa biết Rust → dùng Python/Node |

### 15.2 Rủi ro vận hành

| Rủi ro | Khả năng | Giảm thiểu |
|---|---|---|
| Bảo vệ gõ sai mã thẻ | Trung bình | Đầu đọc USB HID giảm thiểu. Có nút "sửa" |
| Khách quỵt tiền | Thấp | Bảo vệ không mở barrier. Ghi audit |
| Mất thẻ nhiều | Trung bình | Quy trình rõ ràng + phí phạt thẻ mất |
| Bảo vệ tắt app để gian lận | Thấp | Audit log + camera giám sát + manager review |
| Nhân viên nghỉ việc, mang theo kiến thức | Trung bình | Tài liệu hóa + đào tạo chéo |

### 15.3 Rủi ro kinh doanh

| Rủi ro | Khả năng | Giảm thiểu |
|---|---|---|
| Khách không trả tiền license | Trung bình | License key + grace period + suspend |
| Khách đòi custom nhiều | Cao | Scope rõ ràng. Custom = báo giá riêng |
| Support chiếm quá nhiều thời gian | Trung bình | FAQ + video + chat support có giới hạn |
| Cạnh tranh từ giải pháp cloud lớn | Cao | Định vị "on-premise, dữ liệu tại chỗ, chi phí thấp" |

---

## 16. Phụ lục

### 16.1 Checklist trước khi bắt đầu

**Quyết định:**
- [ ] Đã chốt dùng **Tauri (Rust + Vue)** chưa? Hay Python/Node?
- [ ] Bạn đã biết Rust chưa? (Nếu chưa → có 2-3 tháng học trước)
- [ ] Đã chốt mô hình license (desktop + phone-home) chưa?

**Phần cứng:**
- [ ] Đã mua camera Dahua IPC-HFW2239S (3.6mm) chưa?
- [ ] Đã mua đầu đọc RFID USB HID chưa?
- [ ] Đã mua thẻ RFID chưa (bao nhiêu cái)?
- [ ] Đã mua barrier BS-306 chưa?
- [ ] Đã có mini PC chưa?

**Chuẩn bị dev:**
- [ ] Cài Rust (rustup)
- [ ] Cài Node.js + pnpm
- [ ] Cài Tauri CLI (`cargo install tauri-cli`)
- [ ] Cài VS Code + rust-analyzer + Volar (Vue)
- [ ] Tạo repo Git

### 16.2 Câu hỏi mở cần quyết định

| # | Câu hỏi | Lựa chọn |
|---|---|---|
| 1 | Có làm máy in nhiệt trong v1.0 không? | Có / Không |
| 2 | Có làm vé QR (ngoài thẻ RFID) không? | Có / Không |
| 3 | Có làm app mobile cho manager không? | v2.0 |
| 4 | Có làm báo cáo realtime online không? | Không (v1.0 local) |
| 5 | Khách VIP có cần xử lý riêng không? | Có (fee=0) |
| 6 | Có làm đa ngôn ngữ (Anh/Việt) không? | Không (v1.0 tiếng Việt) |

### 16.3 Tài liệu tham khảo

- **Tauri:** https://tauri.app/v2/guides/
- **Rust book:** https://doc.rust-lang.org/book/
- **Vue 3:** https://vuejs.org/guide/
- **rusqlite:** https://docs.rs/rusqlite/
- **serialport:** https://docs.rs/serialport/
- **Dahua RTSP URL format:** tài liệu Dahua (trong hộp sản phẩm)

---

## ✅ Bản spec này đã sẵn sàng để bắt đầu triển khai

**Bước tiếp theo bạn nên làm:**

1. **Đọc lại toàn bộ spec** — note lại chỗ chưa rõ hoặc muốn thay đổi
2. **Trả lời 6 câu hỏi mở** ở mục 16.2
3. **Chốt quyết định Rust vs Python** (dựa trên kinh nghiệm của bạn)
4. **Bắt đầu Milestone 1 (PoC)** — mua 1 bộ phần cứng, thử đọc thẻ + chụp ảnh

Nếu bạn muốn tôi đi sâu vào bất kỳ phần nào — code mẫu Rust chi tiết, schema đầy đủ, UI mockup chi tiết, hay license server FastAPI — chỉ cần nói. Đây là bản đặc tả "sống", sẽ được cập nhật khi bạn phát hiện điểm cần thay đổi trong quá trình làm.
