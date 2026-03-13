CREATE TABLE section_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workout_log_id  UUID NOT NULL REFERENCES workout_logs(id) ON DELETE CASCADE,
    section_id      UUID NOT NULL REFERENCES wod_sections(id) ON DELETE CASCADE,
    finish_time_seconds INTEGER,
    rounds_completed INTEGER,
    extra_reps      INTEGER,
    notes           TEXT,
    is_rx           BOOLEAN NOT NULL DEFAULT true,
    skipped         BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_section_logs_workout_log ON section_logs (workout_log_id);
CREATE INDEX idx_section_logs_section ON section_logs (section_id);

-- Rework workout_exercises to be set-by-set and link to section_logs
ALTER TABLE workout_exercises ADD COLUMN section_log_id UUID REFERENCES section_logs(id) ON DELETE CASCADE;
ALTER TABLE workout_exercises ADD COLUMN set_number INTEGER NOT NULL DEFAULT 1;
ALTER TABLE workout_exercises DROP COLUMN sets;
ALTER TABLE workout_exercises DROP COLUMN workout_log_id;
