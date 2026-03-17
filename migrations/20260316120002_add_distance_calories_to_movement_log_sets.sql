ALTER TABLE movement_log_sets
    ADD COLUMN IF NOT EXISTS distance_meters REAL,
    ADD COLUMN IF NOT EXISTS calories INT;
