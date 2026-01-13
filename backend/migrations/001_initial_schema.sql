-- ============================================
-- miniWiki Database Schema
-- Version: 001
-- Created: 2026-01-11
-- Description: Initial schema for miniWiki Knowledge Management Platform
-- ============================================

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================
-- USERS TABLE
-- ============================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    avatar_url VARCHAR(512),
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
    language VARCHAR(10) NOT NULL DEFAULT 'en',
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_email_verified BOOLEAN NOT NULL DEFAULT false,
    email_verified_at TIMESTAMP,
    last_login_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_active ON users(is_active);

-- ============================================
-- SPACES TABLE
-- ============================================
CREATE TABLE spaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    icon VARCHAR(50),
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_spaces_owner ON spaces(owner_id);
CREATE INDEX idx_spaces_public ON spaces(is_public);

-- ============================================
-- SPACE MEMBERSHIPS TABLE
-- ============================================
CREATE TABLE space_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id UUID NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL DEFAULT 'viewer' CHECK (role IN ('owner', 'editor', 'commenter', 'viewer')),
    joined_at TIMESTAMP NOT NULL DEFAULT NOW(),
    invited_by UUID NOT NULL REFERENCES users(id),
    UNIQUE(space_id, user_id)
);

CREATE INDEX idx_memberships_space ON space_memberships(space_id);
CREATE INDEX idx_memberships_user ON space_memberships(user_id);

-- ============================================
-- DOCUMENTS TABLE
-- ============================================
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id UUID NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES documents(id) ON DELETE SET NULL,
    title VARCHAR(200) NOT NULL,
    icon VARCHAR(50),
    content JSONB NOT NULL DEFAULT '{}',
    content_size INTEGER NOT NULL DEFAULT 0 CHECK (content_size <= 10485760),
    is_archived BOOLEAN NOT NULL DEFAULT false,
    archived_at TIMESTAMP,
    created_by UUID NOT NULL REFERENCES users(id),
    last_edited_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_documents_space ON documents(space_id);
CREATE INDEX idx_documents_parent ON documents(parent_id);
CREATE INDEX idx_documents_archived ON documents(is_archived);
CREATE INDEX idx_documents_updated ON documents(updated_at DESC);

-- ============================================
-- DOCUMENT VERSIONS TABLE
-- ============================================
CREATE TABLE document_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    content JSONB NOT NULL,
    title VARCHAR(200) NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    change_summary VARCHAR(500),
    UNIQUE(document_id, version_number)
);

CREATE INDEX idx_versions_document ON document_versions(document_id, version_number DESC);

-- ============================================
-- FILES TABLE
-- ============================================
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id UUID NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    document_id UUID REFERENCES documents(id) ON DELETE SET NULL,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    file_name VARCHAR(255) NOT NULL,
    file_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL CHECK (file_size <= 52428800),
    storage_path VARCHAR(512) NOT NULL,
    storage_bucket VARCHAR(50) NOT NULL DEFAULT 'files',
    checksum VARCHAR(64) NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_files_space ON files(space_id);
CREATE INDEX idx_files_document ON files(document_id);
CREATE INDEX idx_files_uploader ON files(uploaded_by);
CREATE INDEX idx_files_checksum ON files(checksum);

-- ============================================
-- COMMENTS TABLE
-- ============================================
CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES users(id),
    content TEXT NOT NULL CHECK (length(content) > 0 AND length(content) <= 5000),
    is_resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_comments_document ON comments(document_id);
CREATE INDEX idx_comments_parent ON comments(parent_id);
CREATE INDEX idx_comments_author ON comments(author_id);
CREATE INDEX idx_comments_resolved ON comments(is_resolved);

-- ============================================
-- AUDIT LOGS TABLE
-- ============================================
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID NOT NULL,
    details JSONB,
    ip_address INET,
    user_agent VARCHAR(500),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_user ON audit_logs(user_id);
CREATE INDEX idx_audit_action ON audit_logs(action);
CREATE INDEX idx_audit_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_created ON audit_logs(created_at DESC);

-- ============================================
-- SYNC SESSIONS TABLE
-- ============================================
CREATE TABLE sync_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    client_id VARCHAR(36) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'idle', 'disconnected')),
    last_sync_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sync_document ON sync_sessions(document_id);
CREATE INDEX idx_sync_user ON sync_sessions(user_id);
CREATE INDEX idx_sync_expires ON sync_sessions(expires_at);

-- ============================================
-- SHARE LINKS TABLE
-- ============================================
CREATE TABLE share_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    created_by UUID NOT NULL REFERENCES users(id),
    token VARCHAR(64) UNIQUE NOT NULL,
    access_code VARCHAR(10),
    expires_at TIMESTAMP,
    permission VARCHAR(20) NOT NULL DEFAULT 'view' CHECK (permission IN ('view', 'comment')),
    access_count INTEGER NOT NULL DEFAULT 0,
    max_access INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_share_token ON share_links(token);
CREATE INDEX idx_share_document ON share_links(document_id);

-- ============================================
-- PASSWORD RESETS TABLE
-- ============================================
CREATE TABLE password_resets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(64) UNIQUE NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    used_at TIMESTAMP,
    ip_address INET,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_reset_token ON password_resets(token);
CREATE INDEX idx_reset_user ON password_resets(user_id);
CREATE INDEX idx_reset_expires ON password_resets(expires_at);

-- ============================================
-- REFRESH TOKENS TABLE
-- ============================================
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(64) UNIQUE NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    ip_address INET,
    user_agent VARCHAR(500),
    is_revoked BOOLEAN NOT NULL DEFAULT false,
    revoked_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refresh_token ON refresh_tokens(token);
CREATE INDEX idx_refresh_user ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_expires ON refresh_tokens(expires_at);

-- ============================================
-- FUNCTIONS
-- ============================================

-- Updated at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply updated_at trigger to relevant tables
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_spaces_updated_at BEFORE UPDATE ON spaces
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_documents_updated_at BEFORE UPDATE ON documents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_comments_updated_at BEFORE UPDATE ON comments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Content size calculation trigger
CREATE OR REPLACE FUNCTION update_content_size()
RETURNS TRIGGER AS $$
BEGIN
    NEW.content_size = LENGTH(NEW.content::TEXT)::INTEGER;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_documents_content_size BEFORE INSERT OR UPDATE ON documents
    FOR EACH ROW EXECUTE FUNCTION update_content_size();

-- ============================================
-- VIEWS
-- ============================================

-- View for active documents in a space
CREATE OR REPLACE VIEW active_documents AS
SELECT d.*, s.name as space_name
FROM documents d
JOIN spaces s ON d.space_id = s.id
WHERE d.is_archived = false;

-- View for user spaces with membership info
CREATE OR REPLACE VIEW user_spaces AS
SELECT 
    s.*,
    sm.role,
    sm.joined_at,
    u.email as owner_email
FROM spaces s
JOIN space_memberships sm ON s.id = sm.space_id
JOIN users u ON s.owner_id = u.id
WHERE sm.user_id = u.id;

-- ============================================
-- POLICIES (Row Level Security)
-- ============================================
-- NOTE: RLS policies are commented out for standard PostgreSQL compatibility.
-- These are Supabase-specific policies that require auth.uid() function.
-- Uncomment when deploying to Supabase.

-- Enable RLS on relevant tables
-- ALTER TABLE users ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE spaces ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE documents ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE comments ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE files ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE space_memberships ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;

-- Policies for spaces (Supabase-specific - commented out)
-- CREATE POLICY "Users can view their own spaces" ON spaces
--     FOR SELECT USING (
--         owner_id IN (SELECT id FROM users WHERE auth.uid() = id)
--         OR id IN (SELECT space_id FROM space_memberships WHERE user_id IN (SELECT id FROM users WHERE auth.uid() = id))
--     );

-- ============================================
-- COMMENTS
-- ============================================
-- Migration includes full-text search support via pg_trgm if available
-- CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Full-text search indexes (uncomment if pg_trgm is enabled)
-- CREATE INDEX idx_documents_search ON documents USING gin (title gin_trgm_ops, content gin_trgm_ops);
-- CREATE INDEX idx_users_search ON users USING gin (display_name gin_trgm_ops, email gin_trgm_ops);
