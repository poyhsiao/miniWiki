-- ============================================
-- miniWiki Database Migration
-- Version: 014
-- Date: 2026-01-18
-- Description: Convert TIMESTAMP columns to TIMESTAMPTZ for compatibility with chrono DateTime<Utc>
-- ============================================

-- Drop all views that depend on timestamp columns
DROP VIEW IF EXISTS active_documents;
DROP VIEW IF EXISTS user_spaces;
DROP VIEW IF EXISTS active_email_verifications;
DROP VIEW IF EXISTS active_refresh_tokens;
DROP VIEW IF EXISTS documents_with_versions;
DROP VIEW IF EXISTS recent_document_activity;

-- Users table
ALTER TABLE users ALTER COLUMN email_verified_at TYPE TIMESTAMPTZ;
ALTER TABLE users ALTER COLUMN last_login_at TYPE TIMESTAMPTZ;
ALTER TABLE users ALTER COLUMN created_at TYPE TIMESTAMPTZ;
ALTER TABLE users ALTER COLUMN updated_at TYPE TIMESTAMPTZ;

-- Spaces table
ALTER TABLE spaces ALTER COLUMN created_at TYPE TIMESTAMPTZ;
ALTER TABLE spaces ALTER COLUMN updated_at TYPE TIMESTAMPTZ;

-- Space memberships table
ALTER TABLE space_memberships ALTER COLUMN joined_at TYPE TIMESTAMPTZ;

-- Documents table
ALTER TABLE documents ALTER COLUMN archived_at TYPE TIMESTAMPTZ;
ALTER TABLE documents ALTER COLUMN created_at TYPE TIMESTAMPTZ;
ALTER TABLE documents ALTER COLUMN updated_at TYPE TIMESTAMPTZ;

-- Document versions table
ALTER TABLE document_versions ALTER COLUMN created_at TYPE TIMESTAMPTZ;

-- Files table
ALTER TABLE files ALTER COLUMN deleted_at TYPE TIMESTAMPTZ;
ALTER TABLE files ALTER COLUMN created_at TYPE TIMESTAMPTZ;

-- Comments table
ALTER TABLE comments ALTER COLUMN resolved_at TYPE TIMESTAMPTZ;
ALTER TABLE comments ALTER COLUMN created_at TYPE TIMESTAMPTZ;
ALTER TABLE comments ALTER COLUMN updated_at TYPE TIMESTAMPTZ;

-- Audit logs table
ALTER TABLE audit_logs ALTER COLUMN created_at TYPE TIMESTAMPTZ;

-- Sync sessions table
ALTER TABLE sync_sessions ALTER COLUMN last_sync_at TYPE TIMESTAMPTZ;
ALTER TABLE sync_sessions ALTER COLUMN expires_at TYPE TIMESTAMPTZ;
ALTER TABLE sync_sessions ALTER COLUMN created_at TYPE TIMESTAMPTZ;

-- Share links table
ALTER TABLE share_links ALTER COLUMN expires_at TYPE TIMESTAMPTZ;
ALTER TABLE share_links ALTER COLUMN created_at TYPE TIMESTAMPTZ;

-- Password resets table
ALTER TABLE password_resets ALTER COLUMN expires_at TYPE TIMESTAMPTZ;
ALTER TABLE password_resets ALTER COLUMN used_at TYPE TIMESTAMPTZ;
ALTER TABLE password_resets ALTER COLUMN created_at TYPE TIMESTAMPTZ;

-- Refresh tokens table
ALTER TABLE refresh_tokens ALTER COLUMN expires_at TYPE TIMESTAMPTZ;
ALTER TABLE refresh_tokens ALTER COLUMN revoked_at TYPE TIMESTAMPTZ;
ALTER TABLE refresh_tokens ALTER COLUMN created_at TYPE TIMESTAMPTZ;

-- Email verifications table (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'email_verifications') THEN
        ALTER TABLE email_verifications ALTER COLUMN expires_at TYPE TIMESTAMPTZ;
        ALTER TABLE email_verifications ALTER COLUMN created_at TYPE TIMESTAMPTZ;
    END IF;
END $$;

-- Chunk uploads table (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'chunk_uploads') THEN
        ALTER TABLE chunk_uploads ALTER COLUMN expires_at TYPE TIMESTAMPTZ;
        ALTER TABLE chunk_uploads ALTER COLUMN created_at TYPE TIMESTAMPTZ;
        ALTER TABLE chunk_uploads ALTER COLUMN assembled_at TYPE TIMESTAMPTZ;
    END IF;
END $$;

-- Server clock persistence table (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'server_clock_persistence') THEN
        ALTER TABLE server_clock_persistence ALTER COLUMN created_at TYPE TIMESTAMPTZ;
        ALTER TABLE server_clock_persistence ALTER COLUMN updated_at TYPE TIMESTAMPTZ;
    END IF;
END $$;

-- Recreate all views with updated timestamp columns
CREATE OR REPLACE VIEW active_documents AS
SELECT d.*, s.name as space_name
FROM documents d
JOIN spaces s ON d.space_id = s.id
WHERE d.is_archived = false;

CREATE OR REPLACE VIEW user_spaces AS
SELECT
    s.*,
    sm.role,
    sm.joined_at,
    u.email as owner_email
FROM spaces s
JOIN space_memberships sm ON s.id = sm.space_id
JOIN users u ON s.owner_id = u.id;

-- Create active_email_verifications view only if the table exists
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'email_verifications') THEN
        EXECUTE 'CREATE OR REPLACE VIEW active_email_verifications AS
                 SELECT * FROM email_verifications
                 WHERE expires_at > NOW() AND verified_at IS NULL';
    END IF;
END $$;

CREATE OR REPLACE VIEW active_refresh_tokens AS
SELECT rt.*, u.email as user_email
FROM refresh_tokens rt
JOIN users u ON rt.user_id = u.id
WHERE rt.is_revoked = false AND rt.expires_at > NOW();

CREATE OR REPLACE VIEW documents_with_versions AS
SELECT
    d.*,
    COUNT(dv.id) AS version_count,
    get_document_children_count(d.id) AS children_count
FROM documents d
LEFT JOIN document_versions dv ON d.id = dv.document_id
GROUP BY d.id;

CREATE OR REPLACE VIEW recent_document_activity AS
SELECT
    d.id AS document_id,
    d.title AS document_title,
    d.space_id,
    s.name AS space_name,
    dv.version_number,
    dv.change_summary,
    u.id AS user_id,
    u.display_name AS user_name,
    dv.created_at
FROM document_versions dv
JOIN documents d ON dv.document_id = d.id
JOIN users u ON dv.created_by = u.id
JOIN spaces s ON d.space_id = s.id
ORDER BY dv.created_at DESC;
