CREATE TABLE wods (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title            TEXT NOT NULL,
    description      TEXT,
    workout_type     TEXT NOT NULL DEFAULT 'fortime',
    time_cap_minutes INTEGER,
    programmed_date  DATE NOT NULL,
    created_by       UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE wod_movements (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wod_id      UUID NOT NULL REFERENCES wods(id) ON DELETE CASCADE,
    exercise_id UUID NOT NULL REFERENCES exercises(id) ON DELETE CASCADE,
    reps        INTEGER,
    sets        INTEGER,
    weight_kg   REAL,
    notes       TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
