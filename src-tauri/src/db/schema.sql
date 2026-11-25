BEGIN;

CREATE TABLE cards (
    id              TEXT PRIMARY KEY,
    card_type       TEXT NOT NULL CHECK(card_type IN ('monthly', 'guest')),
    display_code    TEXT NOT NULL,
    owner_name      TEXT,
    owner_phone     TEXT,
    vehicle_plate   TEXT,
    vehicle_type    TEXT CHECK(vehicle_type IN ('motorbike', 'car', 'truck')),
    valid_from      TIMESTAMP,
    valid_to        TIMESTAMP,
    status          TEXT DEFAULT 'active' CHECK(status IN ('active', 'lost', 'expired', 'blocked')),
    notes           TEXT,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_cards_status ON cards(status);
CREATE INDEX idx_cards_display_code ON cards(display_code);

CREATE TABLE operators (
    id              TEXT PRIMARY KEY,
    username        TEXT UNIQUE NOT NULL,
    password_hash   TEXT NOT NULL,
    full_name       TEXT NOT NULL,
    role            TEXT NOT NULL CHECK(role IN ('staff', 'manager')),
    phone           TEXT,
    is_active       INTEGER DEFAULT 1,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE sessions (
    id                  TEXT PRIMARY KEY,
    card_id             TEXT NOT NULL REFERENCES cards(id),
    lane                TEXT NOT NULL,
    entry_time          TIMESTAMP NOT NULL,
    entry_photo_path    TEXT,
    entry_operator      TEXT,
    exit_time           TIMESTAMP,
    exit_photo_path     TEXT,
    exit_operator       TEXT,
    duration_minutes    INTEGER,
    fee                 INTEGER DEFAULT 0,
    fee_collected       INTEGER DEFAULT 0,
    payment_method      TEXT DEFAULT 'cash' CHECK(payment_method IN ('cash', 'free')),
    status              TEXT DEFAULT 'active' CHECK(status IN ('active', 'completed', 'cancelled', 'lost_card')),
    notes               TEXT,
    created_at          TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_sessions_card_status ON sessions(card_id, status);
CREATE INDEX idx_sessions_status_time ON sessions(status, entry_time DESC);
CREATE INDEX idx_sessions_exit_time ON sessions(exit_time);
-- Mỗi thẻ chỉ có tối đa 1 phiên đang đỗ (chặn race khi 2 làn cùng xác nhận).
CREATE UNIQUE INDEX uq_sessions_active_card ON sessions(card_id) WHERE status = 'active';

CREATE TABLE pricing_rules (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    vehicle_type    TEXT NOT NULL CHECK(vehicle_type IN ('motorbike', 'car', 'truck')),
    minute_from     INTEGER NOT NULL,
    minute_to       INTEGER NOT NULL,
    price_per_unit  INTEGER NOT NULL,
    unit_minutes    INTEGER NOT NULL,
    is_active       INTEGER DEFAULT 1,
    effective_from  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE audit_logs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id      TEXT REFERENCES sessions(id),
    operator_id     TEXT REFERENCES operators(id),
    action          TEXT NOT NULL,
    metadata        TEXT,
    ip_address      TEXT,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_audit_created ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_action ON audit_logs(action);

CREATE TABLE app_config (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Bảng giá mặc định (theo ví dụ trong spec 7.3). Manager chỉnh sau ở M3.
INSERT INTO pricing_rules (vehicle_type, minute_from, minute_to, price_per_unit, unit_minutes) VALUES
    ('motorbike', 0,    120,    5000,   120),
    ('motorbike', 120,  1440,   3000,   60),
    ('motorbike', 1440, 999999, 50000,  1440),
    ('car',       0,    60,     15000,  60),
    ('car',       60,   1440,   10000,  60),
    ('car',       1440, 999999, 200000, 1440),
    ('truck',     0,    60,     15000,  60),
    ('truck',     60,   1440,   10000,  60),
    ('truck',     1440, 999999, 200000, 1440);

PRAGMA user_version = 1;
COMMIT;
