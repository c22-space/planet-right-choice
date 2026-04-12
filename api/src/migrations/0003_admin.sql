-- 0003_admin.sql
-- Admin users, extension user accounts, and per-user settings

CREATE TABLE IF NOT EXISTS admin_users (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  email         TEXT    NOT NULL UNIQUE,
  password_hash TEXT    NOT NULL,             -- bcrypt hash (cost factor 12)
  role          TEXT    NOT NULL DEFAULT 'analyst', -- 'superadmin' | 'analyst'
  last_login_at TEXT,
  created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_admin_users_email ON admin_users(email);

-- Extension user accounts (optional — users can use the extension anonymously)
CREATE TABLE IF NOT EXISTS users (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  email        TEXT    NOT NULL UNIQUE,
  display_name TEXT,
  magic_token  TEXT,                          -- current pending magic-link token (hashed)
  token_expiry TEXT,                          -- ISO-8601 expiry of magic_token
  created_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  last_seen_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_users_email       ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_magic_token ON users(magic_token);

-- Per-user preferences (1:1 with users)
CREATE TABLE IF NOT EXISTS user_settings (
  user_id          INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  banner_enabled   INTEGER NOT NULL DEFAULT 1,    -- show/hide the content-script banner
  currency         TEXT    NOT NULL DEFAULT 'USD',
  data_sharing     INTEGER NOT NULL DEFAULT 1,    -- consent to sending anonymous telemetry
  updated_at       TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
