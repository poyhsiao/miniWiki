-- ============================================
-- miniWiki Users Authentication Tables
-- Version: 002
-- Created: 2026-01-11
-- Description: Add email verification, password reset, and refresh token tables
-- ============================================

-- ============================================
-- EMAIL VERIFICATION TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS email_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(64) UNIQUE NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    verified_at TIMESTAMP,
    ip_address INET,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_verification_token ON email_verifications(token);
CREATE INDEX idx_email_verification_user ON email_verifications(user_id);
CREATE INDEX idx_email_verification_expires ON email_verifications(expires_at);

-- ============================================
-- PASSWORD RESET TABLE (already exists, but confirming structure)
-- ============================================
-- Note: This table already exists in 001_initial_schema.sql
-- Columns: id, user_id, token, expires_at, ip_address, used_at, created_at

-- ============================================
-- REFRESH TOKEN TABLE (already exists, but confirming structure)
-- ============================================
-- Note: This table already exists in 001_initial_schema.sql
-- Columns: id, user_id, token, expires_at, ip_address, user_agent, is_revoked, revoked_at, created_at

-- ============================================
-- FUNCTIONS
-- ============================================

-- Function to generate verification token
CREATE OR REPLACE FUNCTION generate_verification_token()
RETURNS VARCHAR(64) AS $$
    SELECT encode(
        digest(gen_random_bytes(32), 'sha256'),
        'hex'
    );
$$ LANGUAGE SQL SECURITY DEFINER;

-- Function to generate password reset token
CREATE OR REPLACE FUNCTION generate_reset_token()
RETURNS VARCHAR(64) AS $$
    SELECT encode(
        digest(gen_random_bytes(32), 'sha256'),
        'hex'
    );
$$ LANGUAGE SQL SECURITY DEFINER;

-- ============================================
-- TRIGGERS (if needed)
-- ============================================

-- Automatic cleanup of expired tokens
CREATE OR REPLACE FUNCTION cleanup_expired_tokens()
RETURNS void AS $$
BEGIN
    -- Delete expired email verification tokens (older than 24 hours)
    DELETE FROM email_verifications
    WHERE expires_at < NOW() - INTERVAL '24 hours'
      AND verified_at IS NULL;

    -- Delete expired refresh tokens
    UPDATE refresh_tokens SET is_revoked = true, revoked_at = NOW()
    WHERE expires_at < NOW() AND is_revoked = false;

    -- Mark password reset tokens as used for audit trail
    UPDATE password_resets SET used_at = NOW()
    WHERE used_at IS NULL
      AND expires_at < NOW();
END;
$$ LANGUAGE PLPGSQL;

-- Schedule periodic cleanup using pg_cron (if available)
-- Execute cleanup every hour
DO $$
BEGIN
    -- Check if pg_cron extension exists
    IF EXISTS (
        SELECT 1 FROM pg_extension WHERE extname = 'pg_cron'
    ) THEN
        -- Schedule the cleanup job
        PERFORM cron.schedule('cleanup_expired_tokens', '0 * * * *', 'SELECT cleanup_expired_tokens()');
    ELSE
        RAISE NOTICE 'pg_cron extension not available. Skipping scheduled cleanup job. Consider running cleanup_expired_tokens() manually or via external scheduler.';
    END IF;
EXCEPTION
    WHEN OTHERS THEN
        RAISE NOTICE 'Could not schedule cleanup job: %. Consider running cleanup_expired_tokens() manually.', SQLERRM;
END $$;

-- ============================================
-- VIEWS
-- ============================================

-- View for active email verifications
CREATE OR REPLACE VIEW active_email_verifications AS
SELECT * FROM email_verifications
WHERE expires_at > NOW() AND verified_at IS NULL;

-- View for active refresh tokens
CREATE OR REPLACE VIEW active_refresh_tokens AS
SELECT rt.*, u.email as user_email
FROM refresh_tokens rt
JOIN users u ON rt.user_id = u.id
WHERE rt.is_revoked = false AND rt.expires_at > NOW();

-- Composite index for email verifications
CREATE INDEX idx_email_verifications_user_expires_verified ON email_verifications(user_id, expires_at, verified_at)
WHERE verified_at IS NULL;
