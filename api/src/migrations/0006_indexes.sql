-- Performance index for impact savings leaderboard queries
CREATE INDEX IF NOT EXISTS idx_impact_events_saving ON impact_events(saving_co2e_kg DESC);
