-- ============================================
-- Database Performance Indexes
-- Version: 012
-- Created: 2026-01-16
-- Description: Additional indexes for query performance optimization
-- ============================================

-- ============================================
-- Documents Table Indexes
-- ============================================

-- Composite index for listing active documents by space and parent
-- Used by: Document list in space, tree view navigation
CREATE INDEX IF NOT EXISTS idx_documents_space_parent_active 
ON documents(space_id, parent_id) 
WHERE is_archived = false;

-- Index on created_by for user activity queries
-- Used by: User contribution history, recent documents
CREATE INDEX IF NOT EXISTS idx_documents_created_by 
ON documents(created_by, created_at DESC);

-- Index on last_edited_by for editor activity
-- Used by: Document editing history, collaboration stats
CREATE INDEX IF NOT EXISTS idx_documents_last_edited_by 
ON documents(last_edited_by, updated_at DESC);

-- ============================================
-- Comments Table Indexes
-- ============================================

-- Composite index for document comments ordered by creation
-- Used by: Comment list in document, threading
CREATE INDEX IF NOT EXISTS idx_comments_document_created 
ON comments(document_id, created_at DESC);

-- Index for unresolved comments
-- Used by: Open comments list, review workflows
CREATE INDEX IF NOT EXISTS idx_comments_unresolved 
ON comments(document_id, is_resolved, created_at DESC)
WHERE is_resolved = false;

-- ============================================
-- Space Memberships Indexes
-- ============================================

-- Composite index for user's spaces
-- Used by: Space list, sidebar navigation
CREATE INDEX IF NOT EXISTS idx_memberships_user_role 
ON space_memberships(user_id, role);

-- Index for checking space membership quickly
-- Used by: Authorization checks, access control
CREATE INDEX IF NOT EXISTS idx_memberships_space_user_role 
ON space_memberships(space_id, user_id, role);

-- ============================================
-- Files Table Indexes
-- ============================================

-- Composite index for files in space ordered by creation
-- Used by: File list in space, recent uploads
CREATE INDEX IF NOT EXISTS idx_files_space_created 
ON files(space_id, created_at DESC);

-- Index for non-deleted files
-- Used by: File queries excluding soft-deleted
CREATE INDEX IF NOT EXISTS idx_files_not_deleted 
ON files(space_id, is_deleted, created_at DESC)
WHERE is_deleted = false;

-- ============================================
-- Sync Sessions Indexes
-- ============================================

-- Index for active sync sessions
-- Used by: Presence tracking, connection management
CREATE INDEX IF NOT EXISTS idx_sync_active 
ON sync_sessions(document_id, status, expires_at)
WHERE status = 'active';

-- ============================================
-- Audit Logs Indexes
-- ============================================

-- Composite index for user action history
-- Used by: User activity logs, security auditing
CREATE INDEX IF NOT EXISTS idx_audit_user_action 
ON audit_logs(user_id, action, created_at DESC);

-- Index for resource-specific audit trail
-- Used by: Document history, access logs
CREATE INDEX IF NOT EXISTS idx_audit_resource_time 
ON audit_logs(resource_type, resource_id, created_at DESC);

-- ============================================
-- Document Versions Indexes
-- ============================================

-- Index for recent versions of a document
-- Used by: Version history, restore operations
CREATE INDEX IF NOT EXISTS idx_versions_document_created 
ON document_versions(document_id, created_at DESC);

-- ============================================
-- Share Links Indexes
-- ============================================

-- Index for active share links
-- Used by: Share link validation, access checking
CREATE INDEX IF NOT EXISTS idx_share_links_active 
ON share_links(document_id, is_active, expires_at)
WHERE is_active = true;

-- ============================================
-- Full-Text Search Indexes (Optional)
-- ============================================

-- Enable pg_trgm extension if not already enabled
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Full-text search index on document titles
-- Used by: Quick title search
CREATE INDEX IF NOT EXISTS idx_documents_title_search 
ON documents USING gin (title gin_trgm_ops);

-- Full-text search index on document content
-- Used by: Full-text content search
CREATE INDEX IF NOT EXISTS idx_documents_content_search 
ON documents USING gin (content gin_trgm_ops);

-- ============================================
-- Comments Indexes (for search)
-- ============================================

-- Full-text search on comment content
-- Used by: Comment search, discussion find
CREATE INDEX IF NOT EXISTS idx_comments_content_search 
ON comments USING gin (content gin_trgm_ops);
