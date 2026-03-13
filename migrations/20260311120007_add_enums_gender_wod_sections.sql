CREATE TYPE wod_phase AS ENUM ('warmup', 'strength', 'conditioning', 'cooldown', 'optional', 'personal');
CREATE TYPE section_type AS ENUM ('fortime', 'amrap', 'emom', 'strength', 'static');
CREATE TYPE gender AS ENUM ('male', 'female');

ALTER TABLE users ADD COLUMN gender gender;

CREATE TABLE wod_sections (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wod_id          UUID NOT NULL REFERENCES wods(id) ON DELETE CASCADE,
    phase           wod_phase NOT NULL,
    title           TEXT,
    section_type    section_type NOT NULL DEFAULT 'static',
    time_cap_minutes INTEGER,
    rounds          INTEGER,
    notes           TEXT,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_wod_sections_wod ON wod_sections (wod_id);
