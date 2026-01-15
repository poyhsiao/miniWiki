-- ============================================
-- miniWiki Documents Tables - Additional Functions & Optimizations
-- Version: 003
-- Created: 2026-01-12
-- Description: Add document-specific functions, indexes, and performance optimizations
-- ============================================

-- ============================================
-- ADDITIONAL INDEXES FOR PERFORMANCE
-- ============================================

-- Composite index for document listing with hierarchy
CREATE INDEX IF NOT EXISTS idx_documents_space_parent_updated
ON documents(space_id, parent_id, updated_at DESC)
WHERE is_archived = false;

-- Index for documents by creator
CREATE INDEX IF NOT EXISTS idx_documents_created_by
ON documents(created_by, created_at DESC);

-- Index for version listing
CREATE INDEX IF NOT EXISTS idx_document_versions_created
ON document_versions(document_id, created_at DESC);

-- ============================================
-- FUNCTIONS
-- ============================================

-- Function to get document hierarchy path
CREATE OR REPLACE FUNCTION get_document_path(p_document_id UUID)
RETURNS TABLE(id UUID, title VARCHAR, level INT) AS $$
WITH RECURSIVE doc_path AS (
    SELECT d.id, d.title, d.parent_id, 0 AS level
    FROM documents d
    WHERE d.id = p_document_id
    
    UNION ALL
    
    SELECT d.id, d.title, d.parent_id, dp.level + 1
    FROM documents d
    JOIN doc_path dp ON d.id = dp.parent_id
    WHERE dp.level < 100
)
SELECT dp.id, dp.title, dp.level
FROM doc_path dp
ORDER BY dp.level DESC;
$$ LANGUAGE SQL STABLE;

-- Function to calculate content size from JSONB
CREATE OR REPLACE FUNCTION calculate_content_size(content JSONB)
RETURNS INT AS $$
SELECT LENGTH(content::TEXT)::INT;
$$ LANGUAGE SQL IMMUTABLE;

-- Function to get next version number for a document
CREATE OR REPLACE FUNCTION get_next_version_number(document_id UUID)
RETURNS INT AS $$
DECLARE
    next_version INT;
    lock_key BIGINT;
BEGIN
    -- Acquire advisory lock for this document to prevent race conditions
    -- Convert UUID to bigint for advisory lock (use hashtext for stable conversion)
    lock_key := hashtext(document_id::TEXT);
    PERFORM pg_advisory_xact_lock(lock_key);

    SELECT COALESCE(MAX(version_number), 0) + 1 INTO next_version
    FROM document_versions
    WHERE document_id = get_next_version_number.document_id;

    RETURN next_version;
END;
$$ LANGUAGE PLPGSQL;

-- Function to get document children count
CREATE OR REPLACE FUNCTION get_document_children_count(document_id UUID)
RETURNS INT AS $$
DECLARE
    child_count INT;
BEGIN
    SELECT COUNT(*) INTO child_count
    FROM documents
    WHERE parent_id = document_id AND is_archived = false;

    RETURN child_count;
END;
$$ LANGUAGE PLPGSQL STABLE;

-- Function to check if document can be deleted (no children)
CREATE OR REPLACE FUNCTION can_delete_document(document_id UUID)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN get_document_children_count(document_id) = 0;
END;
$$ LANGUAGE PLPGSQL STABLE;

-- Function to archive document and optionally its children
CREATE OR REPLACE FUNCTION archive_document_recursive(document_id UUID, cascade BOOLEAN DEFAULT false)
RETURNS void AS $$
DECLARE
    child_id UUID;
BEGIN
    IF cascade THEN
        -- First, recursively archive all children
        FOR child_id IN
            SELECT id FROM documents
            WHERE parent_id = document_id AND is_archived = false
        LOOP
            PERFORM archive_document_recursive(child_id, true);
        END LOOP;
    END IF;

    -- Then archive the current document
    UPDATE documents
    SET is_archived = true, archived_at = NOW()
    WHERE id = document_id;
END;
$$ LANGUAGE PLPGSQL;

-- Function to restore archived document
CREATE OR REPLACE FUNCTION restore_archived_document(document_id UUID)
RETURNS void AS $$
BEGIN
    UPDATE documents
    SET is_archived = false, archived_at = NULL
    WHERE id = document_id;
END;
$$ LANGUAGE PLPGSQL;

-- Function to create document version with automatic version numbering
CREATE OR REPLACE FUNCTION create_document_version(
    p_document_id UUID,
    p_content JSONB,
    p_title VARCHAR,
    p_created_by UUID,
    p_change_summary VARCHAR DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    new_version_id UUID;
    next_version INT;
BEGIN
    next_version := get_next_version_number(p_document_id);

    INSERT INTO document_versions (
        id,
        document_id,
        version_number,
        content,
        title,
        created_by,
        change_summary
    ) VALUES (
        gen_random_uuid(),
        p_document_id,
        next_version,
        p_content,
        p_title,
        p_created_by,
        p_change_summary
    ) RETURNING id INTO new_version_id;

    RETURN new_version_id;
END;
$$ LANGUAGE PLPGSQL SECURITY DEFINER;

-- Function to restore document to specific version
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

    IF version_content IS NULL THEN
        RAISE EXCEPTION 'Version % not found for document %', p_version_number, p_document_id;
    END IF;

    UPDATE documents
    SET
        content = version_content,
        title = version_title,
        last_edited_by = p_restored_by,
        updated_at = NOW()
    WHERE id = p_document_id;

    PERFORM create_document_version(
        p_document_id,
        version_content,
        version_title,
        p_restored_by,
        'Restored to version ' || p_version_number::VARCHAR
    );
END;
$$ LANGUAGE PLPGSQL SECURITY DEFINER;

-- Function to get document version diff between two versions
CREATE OR REPLACE FUNCTION get_document_version_diff(
    p_document_id UUID,
    p_version_from INT,
    p_version_to INT
)
RETURNS JSONB AS $$
DECLARE
    content_from JSONB;
    content_to JSONB;
BEGIN
    SELECT content INTO content_from
    FROM document_versions
    WHERE document_id = p_document_id AND version_number = p_version_from;

    SELECT content INTO content_to
    FROM document_versions
    WHERE document_id = p_document_id AND version_number = p_version_to;

    RETURN jsonb_build_object(
        'from_version', p_version_from,
        'to_version', p_version_to,
        'from_content', content_from,
        'to_content', content_to
    );
END;
$$ LANGUAGE PLPGSQL STABLE;

-- ============================================
-- TRIGGERS
-- ============================================

-- Trigger to create initial version when document is created
CREATE OR REPLACE FUNCTION create_initial_document_version()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.content IS NOT NULL AND NEW.content != '{}'::JSONB THEN
        INSERT INTO document_versions (
            id,
            document_id,
            version_number,
            content,
            title,
            created_by,
            change_summary
        ) VALUES (
            gen_random_uuid(),
            NEW.id,
            1,
            NEW.content,
            NEW.title,
            NEW.created_by,
            'Initial version'
        );
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE PLPGSQL;

-- Attach trigger to documents table
DROP TRIGGER IF EXISTS trg_create_initial_version ON documents;
CREATE TRIGGER trg_create_initial_version
    AFTER INSERT ON documents
    FOR EACH ROW
    EXECUTE FUNCTION create_initial_document_version();

-- ============================================
-- CLEANUP FUNCTIONS
-- ============================================

-- Function to cleanup old document versions (retention policy)
CREATE OR REPLACE FUNCTION cleanup_old_document_versions(
    p_document_id UUID DEFAULT NULL,
    p_keep_count INT DEFAULT 10
)
RETURNS INT AS $$
DECLARE
    deleted_count INT := 0;
    v_document_id UUID;
BEGIN
    IF p_document_id IS NULL THEN
        FOR v_document_id IN SELECT DISTINCT document_id FROM document_versions LOOP
            deleted_count := deleted_count + cleanup_old_document_versions(v_document_id, p_keep_count);
        END LOOP;
    ELSE
        DELETE FROM document_versions
        WHERE document_id = p_document_id
        AND id NOT IN (
            SELECT id FROM document_versions
            WHERE document_id = p_document_id
            ORDER BY version_number DESC
            LIMIT p_keep_count
        );

        GET DIAGNOSTICS deleted_count = ROW_COUNT;
    END IF;

    RETURN deleted_count;
END;
$$ LANGUAGE PLPGSQL;

-- ============================================
-- VIEWS
-- ============================================

-- View for document with version count
CREATE OR REPLACE VIEW documents_with_versions AS
SELECT
    d.*,
    COUNT(dv.id) AS version_count,
    get_document_children_count(d.id) AS children_count
FROM documents d
LEFT JOIN document_versions dv ON d.id = dv.document_id
GROUP BY d.id;

-- View for recent document activity
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
ORDER BY dv.created_at DESC
LIMIT 100;

-- ============================================
-- RLS POLICIES FOR DOCUMENTS
-- Note: auth extension not available, using stored procedures for access control
-- ============================================

-- Enable RLS on documents table
ALTER TABLE documents ENABLE ROW LEVEL SECURITY;

-- Note: RLS policies require auth extension or will be handled at application level
-- For now, access control is implemented in the document_service handlers

-- Create a permissive policy for development/testing
-- IMPORTANT: In production, replace this with proper authorization policies
-- that check space_memberships and user permissions
-- SECURITY: This permissive policy is only allowed in non-production environments
-- The migration runner should fail if this policy exists when NODE_ENV=production
DROP POLICY IF EXISTS documents_allow_all ON documents;
CREATE POLICY documents_allow_all ON documents
    FOR ALL
    USING (true)
    WITH CHECK (true);

-- Pre-flight check: Ensure this migration doesn't run in production
-- Comment out the following line after proper RLS policies are implemented
-- DO $$
-- BEGIN
--     IF current_setting('app.environment', true) = 'production' THEN
--         RAISE EXCEPTION 'CRITICAL: documents_allow_all policy detected in production. Replace with proper RLS policies before deployment.';
--     END IF;
-- END;
-- $$;

-- TODO: Replace the above policy with proper RLS policies such as:
-- CREATE POLICY documents_select_policy ON documents
--     FOR SELECT
--     USING (
--         EXISTS (
--             SELECT 1 FROM space_memberships sm
--             WHERE sm.space_id = documents.space_id
--             AND sm.user_id = current_setting('app.current_user_id')::UUID
--         )
--     );
--
-- Similar policies should be created for INSERT, UPDATE, DELETE operations

