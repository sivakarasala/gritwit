-- Per-set data for movement logs (e.g., "9-8-7-6" stores 4 rows with individual reps/weights)
CREATE TABLE IF NOT EXISTS movement_log_sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    movement_log_id UUID NOT NULL REFERENCES movement_logs(id) ON DELETE CASCADE,
    set_number INT NOT NULL,
    reps INT,
    weight_kg REAL,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_movement_log_sets_movement_log_id ON movement_log_sets(movement_log_id);
