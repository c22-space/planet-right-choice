-- 0001_initial.sql
-- Core schema: categories, products, fp_detections, estimations, rate_limits

CREATE TABLE IF NOT EXISTS categories (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  slug           TEXT    NOT NULL UNIQUE,
  name           TEXT    NOT NULL,
  parent_id      INTEGER REFERENCES categories(id),
  avg_co2e_kg    REAL,
  avg_co2e_scope TEXT,           -- 'lifecycle' | 'cradle-to-gate' | 'use-phase'
  factor_source  TEXT,           -- e.g. "ecoinvent-3.9", "USEEIO-v2.1"
  created_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_categories_slug   ON categories(slug);
CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);

CREATE TABLE IF NOT EXISTS products (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  name             TEXT    NOT NULL,
  brand            TEXT,
  category_id      INTEGER REFERENCES categories(id),
  asin             TEXT    UNIQUE,                   -- Amazon ASIN if applicable
  url              TEXT,                             -- canonical product URL
  image_url        TEXT,                             -- product image URL or R2 key
  description      TEXT,
  co2e_kg          REAL    NOT NULL,
  co2e_scope       TEXT    NOT NULL DEFAULT 'lifecycle', -- 'lifecycle' | 'cradle-to-gate' | 'use-phase'
  co2e_source      TEXT    NOT NULL DEFAULT 'estimate',  -- 'fp_tag' | 'certified' | 'manual' | 'estimated'
  co2e_confidence  REAL    NOT NULL DEFAULT 0.5,         -- 0.0–1.0
  certifications   TEXT,                                 -- JSON array of strings
  materials        TEXT,                                 -- JSON array: [{name, fraction}]
  weight_kg        REAL,
  origin_country   TEXT,                                -- ISO 3166-1 alpha-2
  is_active        INTEGER NOT NULL DEFAULT 1,
  created_at       TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at       TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_products_category ON products(category_id);
CREATE INDEX IF NOT EXISTS idx_products_asin     ON products(asin);
CREATE INDEX IF NOT EXISTS idx_products_active   ON products(is_active);
CREATE INDEX IF NOT EXISTS idx_products_co2e     ON products(co2e_kg);

-- fp: tag detections — one row per page scan where fp: tags were found
-- page URLs are NEVER stored in plaintext; only SHA-256 hex digest
CREATE TABLE IF NOT EXISTS fp_detections (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id      TEXT    NOT NULL,            -- random UUID, not linked to a user
  domain          TEXT    NOT NULL,
  page_url_hash   TEXT    NOT NULL,            -- SHA-256 hex of the full URL
  product_name    TEXT,
  co2e_kg         REAL,
  co2e_scope      TEXT,                        -- scope from fp: tag
  fp_version      TEXT,                        -- fp:version tag value if present
  raw_tags        TEXT,                        -- JSON object of all fp: meta tag values
  detected_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_fp_detections_domain  ON fp_detections(domain);
CREATE INDEX IF NOT EXISTS idx_fp_detections_session ON fp_detections(session_id);
CREATE INDEX IF NOT EXISTS idx_fp_detections_date    ON fp_detections(detected_at);

-- Estimation results — one row per page scan where no fp: tags were found
CREATE TABLE IF NOT EXISTS estimations (
  id                INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id        TEXT    NOT NULL,
  domain            TEXT    NOT NULL,
  page_url_hash     TEXT    NOT NULL,          -- SHA-256 hex
  category_id       INTEGER REFERENCES categories(id),
  product_name      TEXT,
  signals           TEXT    NOT NULL,          -- JSON: PageSignals snapshot
  estimated_co2e_kg REAL    NOT NULL,
  confidence        REAL    NOT NULL,          -- 0.0–1.0, capped at 0.88
  tier              INTEGER NOT NULL,          -- 1 | 2 | 3
  method_version    TEXT    NOT NULL,          -- semver for re-scoring later
  created_at        TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_estimations_domain  ON estimations(domain);
CREATE INDEX IF NOT EXISTS idx_estimations_session ON estimations(session_id);
CREATE INDEX IF NOT EXISTS idx_estimations_tier    ON estimations(tier);
CREATE INDEX IF NOT EXISTS idx_estimations_date    ON estimations(created_at);

-- Simple token-bucket rate limiting
CREATE TABLE IF NOT EXISTS rate_limits (
  key         TEXT    PRIMARY KEY,            -- e.g. "apikey:abc123"
  tokens      INTEGER NOT NULL DEFAULT 100,
  last_refill TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
