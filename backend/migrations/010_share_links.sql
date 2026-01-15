-- Migration: 010_share_links
-- Purpose: Create share_links table for public document sharing
-- Created: 2026-01-15

-- Create share_links table for public document access via share tokens
CREATE TABLE IF NOT EXISTS share_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    created_by UUID NOT NULL REFERENCES users(id),
    token VARCHAR(64) NOT NULL UNIQUE,
    access_code VARCHAR(255),  -- Stores bcrypt hash of access code, not plain text
    expires_at TIMESTAMP WITH TIME ZONE,
    permission VARCHAR(20) NOT NULL DEFAULT 'view' CHECK (permission IN ('view', 'comment')),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    click_count INTEGER NOT NULL DEFAULT 0,
    max_access_count INTEGER
);

-- Index for efficient token lookup
CREATE INDEX IF NOT EXISTS idx_share_links_token ON share_links(token);

-- Index for document-based share link lookup
CREATE INDEX IF NOT EXISTS idx_share_links_document ON share_links(document_id);

-- Index for active shares query
CREATE INDEX IF NOT EXISTS idx_share_links_active ON share_links(is_active, expires_at);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_share_links_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
DROP TRIGGER IF EXISTS trigger_share_links_updated_at ON share_links;
CREATE TRIGGER trigger_share_links_updated_at
    BEFORE UPDATE ON share_links
    FOR EACH ROW
    EXECUTE FUNCTION update_share_links_updated_at();

-- Comments for RLS (Row Level Security)
COMMENT ON TABLE share_links IS 'Public share links for external document access';
COMMENT ON COLUMN share_links.token IS 'Share token for URL generation';
COMMENT ON COLUMN share_links.access_code IS 'Optional password for protected shares (4-10 chars)';
COMMENT ON COLUMN share_links.permission IS 'Access level: view or comment';
COMMENT ON COLUMN share_links.max_access_count IS 'Optional limit on number of accesses';
