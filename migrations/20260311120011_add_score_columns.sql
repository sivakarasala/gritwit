-- Add scoring columns to section_logs
ALTER TABLE section_logs ADD COLUMN weight_kg REAL;
ALTER TABLE section_logs ADD COLUMN score_value INTEGER;

-- One score per section per workout log
ALTER TABLE section_logs ADD CONSTRAINT uq_section_logs_workout_section
    UNIQUE (workout_log_id, section_id);

-- Leaderboard index: fast lookups by section, rx status, ranked by score
CREATE INDEX idx_section_logs_leaderboard
    ON section_logs (section_id, is_rx, score_value)
    WHERE score_value IS NOT NULL;

-- One WOD log per user per date (prevent duplicate submissions)
CREATE UNIQUE INDEX uq_workout_logs_user_wod_date
    ON workout_logs (user_id, wod_id, workout_date)
    WHERE wod_id IS NOT NULL;

-- Gym-wide queries: all logs for a WOD on a given date
CREATE INDEX idx_workout_logs_wod_date
    ON workout_logs (wod_id, workout_date)
    WHERE wod_id IS NOT NULL;
