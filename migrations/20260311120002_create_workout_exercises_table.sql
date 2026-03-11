CREATE TABLE workout_exercises (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workout_log_id UUID NOT NULL REFERENCES workout_logs(id) ON DELETE CASCADE,
    exercise_id UUID NOT NULL REFERENCES exercises(id),
    sets INT,
    reps INT,
    weight_kg REAL,
    duration_seconds INT,
    sort_order INT NOT NULL DEFAULT 0,
    notes TEXT
);

CREATE INDEX idx_workout_exercises_log ON workout_exercises (workout_log_id);
