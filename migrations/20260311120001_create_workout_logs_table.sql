CREATE TABLE workout_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workout_date DATE NOT NULL DEFAULT CURRENT_DATE,
    workout_type TEXT NOT NULL,
    name TEXT,
    notes TEXT,
    duration_seconds INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_workout_logs_date ON workout_logs (workout_date DESC);
