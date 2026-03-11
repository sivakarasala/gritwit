CREATE TABLE exercises (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    movement_type TEXT,
    muscle_groups TEXT[] DEFAULT '{}',
    description TEXT,
    demo_video_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_exercises_name ON exercises (LOWER(name));
