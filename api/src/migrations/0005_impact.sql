-- Impact tracking: records every time a user clicks a recommended alternative,
-- capturing the CO2e saving from the baseline to the chosen product.
CREATE TABLE IF NOT EXISTS impact_events (
  id                  INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id          TEXT    NOT NULL,
  domain              TEXT    NOT NULL,
  baseline_co2e_kg    REAL    NOT NULL,
  alternative_co2e_kg REAL    NOT NULL,
  saving_co2e_kg      REAL    NOT NULL GENERATED ALWAYS AS (baseline_co2e_kg - alternative_co2e_kg) STORED,
  baseline_source     TEXT    NOT NULL CHECK(baseline_source IN ('fp_tag','estimated','manual','certified')),
  baseline_tier       INTEGER,            -- null if fp: tag was the source
  alternative_id      INTEGER REFERENCES products(id),
  category_slug       TEXT,
  country_code        TEXT,
  created_at          TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Aggregate snapshot refreshed by a cron job (or computed on-demand for admin)
CREATE TABLE IF NOT EXISTS impact_totals (
  id                  INTEGER PRIMARY KEY AUTOINCREMENT,
  total_events        INTEGER NOT NULL DEFAULT 0,
  total_saving_co2e_kg REAL   NOT NULL DEFAULT 0,
  unique_sessions     INTEGER NOT NULL DEFAULT 0,
  period_start        TEXT    NOT NULL,
  period_end          TEXT    NOT NULL,
  computed_at         TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_impact_events_session  ON impact_events(session_id);
CREATE INDEX IF NOT EXISTS idx_impact_events_date     ON impact_events(created_at);
CREATE INDEX IF NOT EXISTS idx_impact_events_category ON impact_events(category_slug);
