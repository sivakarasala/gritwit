-- Add gender column to users for prescribing correct weights.
-- Values: 'male', 'female', or NULL (not set yet).
ALTER TABLE users ADD COLUMN IF NOT EXISTS gender TEXT;
