-- Migration: Create spaces table
-- Part of User Story 2: Document Organization

CREATE TABLE IF NOT EXISTS spaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    icon VARCHAR(50),
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Indexes for spaces
CREATE INDEX IF NOT EXISTS idx_spaces_owner ON spaces(owner_id);
CREATE INDEX IF NOT EXISTS idx_spaces_public ON spaces(is_public);
CREATE INDEX IF NOT EXISTS idx_spaces_name ON spaces(name);

-- Comments
COMMENT ON TABLE spaces IS 'Represents a workspace/collection of documents (analogous to Notion pages or Confluence spaces)';
COMMENT ON COLUMN spaces.is_public IS 'When true, space is visible to anyone with link';
