-- ============================================
-- ADD RESTORED_FROM_VERSION TO DOCUMENT_VERSIONS
-- Migration: 007_add_restored_from_version.sql
-- ============================================
-- Adds the restored_from_version field to track when a version
-- was created by restoring from another version.
--
-- This field enables:
-- 1. Audit trail of restore operations
-- 2. Prevention of circular restore chains
-- 3. Clear history of version relationships
-- ============================================

-- UP: Add restored_from_version column
ALTER TABLE document_versions
ADD COLUMN IF NOT EXISTS restored_from_version INTEGER;

-- Add comment for documentation
COMMENT ON COLUMN document_versions.restored_from_version IS
    'When version was created by restoring from another version (version number restored from)';

-- Create index for efficient queries on restore chains
CREATE INDEX IF NOT EXISTS idx_versions_restored_from
ON document_versions(restored_from_version, document_id)
WHERE restored_from_version IS NOT NULL;
