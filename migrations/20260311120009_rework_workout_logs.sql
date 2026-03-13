ALTER TABLE workout_logs ADD COLUMN wod_id UUID REFERENCES wods(id) ON DELETE SET NULL;

ALTER TABLE workout_logs DROP COLUMN workout_type;
ALTER TABLE workout_logs DROP COLUMN name;
ALTER TABLE workout_logs DROP COLUMN duration_seconds;

CREATE INDEX idx_workout_logs_wod ON workout_logs (wod_id);
