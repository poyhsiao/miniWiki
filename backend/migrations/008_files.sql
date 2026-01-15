-- Migration: Create files table
-- Purpose: File attachments for documents
-- Created: 2026-01-15

-- Create files table
CREATE TABLE IF NOT EXISTS files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id UUID NOT NULL REFERENCES spaces(id),
    document_id UUID NULL REFERENCES documents(id),
    uploaded_by UUID NOT NULL REFERENCES users(id),
    file_name VARCHAR(255) NOT NULL,
    file_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    storage_path VARCHAR(512) NOT NULL,
    storage_bucket VARCHAR(50) NOT NULL DEFAULT 'files',
    checksum VARCHAR(64) NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_files_space ON files(space_id);
CREATE INDEX IF NOT EXISTS idx_files_document ON files(document_id);
CREATE INDEX IF NOT EXISTS idx_files_uploader ON files(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_files_checksum ON files(checksum);
CREATE INDEX IF NOT EXISTS idx_files_created ON files(created_at DESC);

-- Add constraints
ALTER TABLE files ADD CONSTRAINT chk_file_size CHECK (file_size <= 52428800); -- 50MB max

ALTER TABLE files ADD CONSTRAINT chk_file_type CHECK (
    file_type LIKE 'image/%'
    OR file_type LIKE 'application/pdf'
    OR file_type LIKE 'text/%'
    OR file_type LIKE 'video/%'
    OR file_type LIKE 'audio/%'
    OR file_type = 'application/zip'
);

-- Note: chunked_uploads table is created in migration 009_chunked_uploads.sql
-- Do not create it here to avoid conflicts

-- Create function to soft delete file
CREATE OR REPLACE FUNCTION soft_delete_file(p_file_id UUID)
RETURNS void AS $$
BEGIN
    UPDATE files
    SET is_deleted = true,
        deleted_at = NOW()
    WHERE id = p_file_id;
END;
$$ LANGUAGE plpgsql;

-- Create function to restore soft-deleted file
CREATE OR REPLACE FUNCTION restore_file(p_file_id UUID)
RETURNS void AS $$
BEGIN
    UPDATE files
    SET is_deleted = false,
        deleted_at = NULL
    WHERE id = p_file_id AND deleted_at > NOW() - INTERVAL '30 days';
END;
$$ LANGUAGE plpgsql;

-- Create function to permanently delete file
CREATE OR REPLACE FUNCTION permanent_delete_file(p_file_id UUID)
RETURNS void AS $$
DECLARE
    v_storage_path VARCHAR(512);
BEGIN
    -- Get storage path before deletion
    SELECT storage_path INTO v_storage_path
    FROM files WHERE id = p_file_id;

    -- Delete from database
    DELETE FROM files WHERE id = p_file_id;

    -- Note: Actual file in MinIO should be deleted by a background job
    -- This prevents immediate data loss and allows for recovery
    RAISE NOTICE 'File % marked for deletion from storage', v_storage_path;
END;
$$ LANGUAGE plpgsql;

-- Grant permissions (adjust as needed for your setup)
-- Note: These tables use UUID primary keys (gen_random_uuid()), not sequences
-- GRANT SELECT, INSERT, UPDATE, DELETE ON files TO miniwiki;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON chunked_uploads TO miniwiki;

-- Down migration
-- DROP FUNCTION IF EXISTS cleanup_expired_chunked_uploads();
-- DROP TRIGGER IF EXISTS trigger_cleanup_chunked_uploads ON chunked_uploads;
-- DROP FUNCTION IF EXISTS soft_delete_file(UUID);
-- DROP FUNCTION IF EXISTS restore_file(UUID);
-- DROP FUNCTION IF EXISTS permanent_delete_file(UUID);
-- DROP TABLE IF EXISTS chunked_uploads;
-- DROP TABLE IF EXISTS files;
