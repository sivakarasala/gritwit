-- Re-add workout_log_id for custom (non-WOD) exercise logging.
-- A workout_exercise belongs to EITHER a section_log (WOD score) OR a workout_log (custom log).
ALTER TABLE workout_exercises
    ADD COLUMN workout_log_id UUID REFERENCES workout_logs(id) ON DELETE CASCADE;

CREATE INDEX idx_workout_exercises_workout_log ON workout_exercises (workout_log_id)
    WHERE workout_log_id IS NOT NULL;

-- Ensure each exercise row links to at least one parent
ALTER TABLE workout_exercises
    ADD CONSTRAINT chk_workout_exercise_parent
    CHECK (section_log_id IS NOT NULL OR workout_log_id IS NOT NULL);
