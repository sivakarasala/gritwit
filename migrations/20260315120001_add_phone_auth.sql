-- Add phone number column and OTP codes table for SMS login

-- Make email nullable (phone-only users won't have one initially)
ALTER TABLE users ALTER COLUMN email DROP NOT NULL;

-- Add phone column (E.164 format, e.g. +919876543210)
ALTER TABLE users ADD COLUMN phone TEXT UNIQUE;

-- Index for phone lookups
CREATE INDEX idx_users_phone ON users (phone) WHERE phone IS NOT NULL;

-- OTP codes table for verification
CREATE TABLE otp_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone TEXT NOT NULL,
    code TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Clean up expired codes automatically via index (app deletes on verify)
CREATE INDEX idx_otp_codes_phone ON otp_codes (phone, created_at DESC);
