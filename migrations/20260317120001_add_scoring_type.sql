-- Add scoring_type to exercises table
ALTER TABLE exercises
    ADD COLUMN scoring_type TEXT NOT NULL DEFAULT 'weight_and_reps';

-- Apply defaults based on category
UPDATE exercises SET scoring_type = 'reps_only'
    WHERE category IN ('gymnastics', 'calisthenics', 'plyometrics', 'warmup');

UPDATE exercises SET scoring_type = 'distance'
    WHERE category IN ('cardio', 'conditioning');

UPDATE exercises SET scoring_type = 'time'
    WHERE category IN ('yoga', 'mobility', 'meditation', 'breathing', 'chanting', 'cooldown', 'sports');

-- weightlifting, powerlifting, strongman, bodybuilding remain 'weight_and_reps' (the default)

-- Add distance and calorie tracking to custom workout sets
ALTER TABLE workout_exercises
    ADD COLUMN distance_meters REAL,
    ADD COLUMN calories        INTEGER;

-- Add distance and calorie tracking to WOD per-set logs
-- (movement_logs is intentionally excluded — movement_log_sets is the single source of truth)
ALTER TABLE movement_log_sets
    ADD COLUMN distance_meters REAL,
    ADD COLUMN calories        INTEGER;
