-- Stores per-movement results within a section log.
-- Allows users to track actual weight/reps/sets for each movement in a WOD section.
CREATE TABLE IF NOT EXISTS movement_logs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    section_log_id UUID NOT NULL REFERENCES section_logs(id) ON DELETE CASCADE,
    movement_id    UUID NOT NULL REFERENCES wod_movements(id) ON DELETE CASCADE,
    reps        INTEGER,
    sets        INTEGER,
    weight_kg   REAL,
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_movement_logs_section_log ON movement_logs(section_log_id);
