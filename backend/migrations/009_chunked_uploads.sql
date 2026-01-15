-- Migration: Create chunked_uploads table for file upload sessions
-- Created for Phase 13 - File Attachments feature

CREATE TABLE IF NOT EXISTS chunked_uploads (
    upload_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id UUID NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    document_id UUID REFERENCES documents(id) ON DELETE CASCADE,
    file_name VARCHAR(500) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    total_size BIGINT NOT NULL,
    chunk_size BIGINT NOT NULL,
    total_chunks INTEGER NOT NULL,
    uploaded_chunks INTEGER[] DEFAULT ARRAY[]::INTEGER[],
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Trigger function to cleanup orphaned file chunks before deletion
CREATE OR REPLACE FUNCTION cleanup_chunked_uploads_files()
RETURNS TRIGGER AS $$
BEGIN
    -- In a production system, you would call storage cleanup here:
    -- PERFORM delete_from_storage(OLD.file_name, OLD.upload_id);
    -- For now, we just log the cleanup
    -- The actual file cleanup should be handled by:
    -- 1. A scheduled job that scans storage for files with non-existent upload_ids
    -- 2. Or application-layer code that calls storage deletion before DB deletion
    -- 3. Or a background worker that processes cleanup jobs
    RAISE LOG 'Cleanup needed for chunked upload: %, file: %', OLD.upload_id, OLD.file_name;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS cleanup_chunked_uploads_files_trigger ON chunked_uploads;
CREATE TRIGGER cleanup_chunked_uploads_files_trigger
BEFORE DELETE ON chunked_uploads
FOR EACH ROW
EXECUTE FUNCTION cleanup_chunked_uploads_files();

-- Index for querying by space_id
CREATE INDEX IF NOT EXISTS idx_chunked_uploads_space_id
    ON chunked_uploads(space_id);

-- Composite index for cleanup queries filtering by expires_at
CREATE INDEX IF NOT EXISTS idx_chunked_uploads_expires_at
    ON chunked_uploads(expires_at);
