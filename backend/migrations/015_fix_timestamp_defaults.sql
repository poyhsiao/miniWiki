-- ============================================
-- miniWiki Database Migration
-- Version: 015
-- Date: 2026-01-18
-- Description: Fix TIMESTAMP vs TIMESTAMPTZ inconsistencies by replacing NOW() with CURRENT_TIMESTAMP
-- ============================================

-- Fix DEFAULT values in tables
ALTER TABLE users ALTER COLUMN created_at SET DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC';
ALTER TABLE users ALTER COLUMN updated_at SET DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC';

ALTER TABLE spaces ALTER COLUMN created_at SET DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC';
ALTER TABLE spaces ALTER COLUMN updated_at SET DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC';

ALTER TABLE space_memberships ALTER COLUMN joined_at SET DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC';

ALTER TABLE documents ALTER COLUMN created_at SET DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC';
ALTER TABLE documents ALTER COLUMN updated_at SET DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC';

ALTER TABLE document_versions ALTER COLUMN created_at SET DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC';

-- deleted_at should remain nullable with no default (set explicitly when is_deleted = true)
ALTER TABLE files ALTER COLUMN deleted_at DROP DEFAULT;
ALTER TABLE files ALTER COLUMN created_at SET DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC';

-- resolved_at should remain nullable with no default (set explicitly when is_resolved = true)
ALTER TABLE comments ALTER COLUMN resolved_at DROP DEFAULT;
ALTER TABLE comments ALTER COLUMN created_at SET DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC';
ALTER TABLE comments ALTER COLUMN updated_at SET DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC';

-- Fix functions that use NOW()
-- Users function
CREATE OR REPLACE FUNCTION update_user_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Documents function (from 001_documents.sql in document_service)
CREATE OR REPLACE FUNCTION update_document_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- archive_document_recursive function (from 003_documents.sql)
CREATE OR REPLACE FUNCTION archive_document_recursive(document_id UUID, cascade BOOLEAN DEFAULT false)
RETURNS void AS $$
DECLARE
    child_id UUID;
BEGIN
    IF cascade THEN
        FOR child_id IN
            SELECT id FROM documents
            WHERE parent_id = document_id AND is_archived = false
        LOOP
            PERFORM archive_document_recursive(child_id, true);
        END LOOP;
    END IF;

    UPDATE documents
    SET is_archived = true, archived_at = CURRENT_TIMESTAMP
    WHERE id = document_id;
END;
$$ LANGUAGE PLPGSQL;

-- restore_document_to_version function (from 003_documents.sql)
CREATE OR REPLACE FUNCTION restore_document_to_version(
    p_document_id UUID,
    p_version_number INT,
    p_restored_by UUID
)
RETURNS void AS $$
DECLARE
    version_content JSONB;
    version_title VARCHAR;
BEGIN
    SELECT content, title INTO version_content, version_title
    FROM document_versions
    WHERE document_id = p_document_id AND version_number = p_version_number;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Version % not found for document %', p_version_number, p_document_id;
    END IF;

    UPDATE documents
    SET
        content = version_content,
        title = version_title,
        last_edited_by = p_restored_by,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = p_document_id;

    PERFORM create_document_version(
        p_document_id,
        version_content,
        version_title,
        p_restored_by,
        'Restored to version ' || p_version_number::VARCHAR
    );
END;
$$ LANGUAGE PLPGSQL SECURITY DEFINER SET search_path = public;
