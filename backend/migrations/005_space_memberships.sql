-- Migration: Create space_memberships table
-- Part of User Story 2: Document Organization

CREATE TABLE IF NOT EXISTS space_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id UUID NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('owner', 'editor', 'commenter', 'viewer')),
    joined_at TIMESTAMP NOT NULL DEFAULT NOW(),
    invited_by UUID REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(space_id, user_id)
);

-- Indexes for space_memberships
CREATE INDEX IF NOT EXISTS idx_space_memberships_space ON space_memberships(space_id);
CREATE INDEX IF NOT EXISTS idx_space_memberships_user ON space_memberships(user_id);
CREATE INDEX IF NOT EXISTS idx_space_memberships_role ON space_memberships(role);

-- Comments
COMMENT ON TABLE space_memberships IS 'Junction table linking users to spaces with roles';
COMMENT ON COLUMN space_memberships.role IS 'Member role: owner, editor, commenter, or viewer';

-- Constraint: owner role cannot be removed or changed
-- This is enforced at application level
