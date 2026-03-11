ALTER TABLE workout_logs ADD COLUMN user_id UUID REFERENCES users(id);
ALTER TABLE exercises ADD COLUMN created_by UUID REFERENCES users(id);

CREATE INDEX idx_workout_logs_user ON workout_logs (user_id);
CREATE INDEX idx_exercises_created_by ON exercises (created_by);
