-- The old index blocks re-adding a soft-deleted exercise with the same name.
-- Replace it with a partial index that only covers non-deleted rows.
DROP INDEX IF EXISTS idx_exercises_name;
CREATE UNIQUE INDEX idx_exercises_name ON exercises (LOWER(name)) WHERE deleted_at IS NULL;
