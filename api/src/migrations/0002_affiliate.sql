-- 0002_affiliate.sql
-- Affiliate link rewriting: rules, click events, revenue snapshots

CREATE TABLE IF NOT EXISTS affiliate_rules (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  source_asin   TEXT    NOT NULL,              -- the ASIN to intercept
  target_asin   TEXT,                          -- replacement ASIN (NULL = same ASIN, tag-only)
  affiliate_tag TEXT    NOT NULL,              -- Amazon Associate tag
  priority      INTEGER NOT NULL DEFAULT 0,   -- higher wins on conflict
  reason        TEXT,                          -- admin note: why this rule exists
  is_active     INTEGER NOT NULL DEFAULT 1,
  created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_affiliate_rules_source ON affiliate_rules(source_asin) WHERE is_active = 1;
CREATE INDEX IF NOT EXISTS idx_affiliate_rules_active ON affiliate_rules(is_active);

-- Click events — fired when the extension rewrites an Amazon URL
CREATE TABLE IF NOT EXISTS affiliate_clicks (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  rule_id      INTEGER REFERENCES affiliate_rules(id), -- NULL if no matching rule (default tag applied)
  source_asin  TEXT    NOT NULL,
  target_asin  TEXT,                           -- NULL if same as source
  session_id   TEXT    NOT NULL,
  country_code TEXT,                           -- ISO 3166-1 alpha-2 from CF-IPCountry header
  clicked_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_affiliate_clicks_rule    ON affiliate_clicks(rule_id);
CREATE INDEX IF NOT EXISTS idx_affiliate_clicks_date    ON affiliate_clicks(clicked_at);
CREATE INDEX IF NOT EXISTS idx_affiliate_clicks_country ON affiliate_clicks(country_code);

-- Aggregated revenue snapshots synced from Amazon PA-API (run as a scheduled Worker cron)
CREATE TABLE IF NOT EXISTS affiliate_revenue (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  period_year   INTEGER NOT NULL,
  period_month  INTEGER NOT NULL,
  asin          TEXT,                          -- NULL = all-up total
  gross_usd     REAL    NOT NULL DEFAULT 0,
  net_usd       REAL    NOT NULL DEFAULT 0,
  clicks        INTEGER NOT NULL DEFAULT 0,
  orders        INTEGER NOT NULL DEFAULT 0,
  items_shipped INTEGER NOT NULL DEFAULT 0,
  synced_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  UNIQUE(period_year, period_month, asin)
);

CREATE INDEX IF NOT EXISTS idx_affiliate_revenue_period ON affiliate_revenue(period_year, period_month);
