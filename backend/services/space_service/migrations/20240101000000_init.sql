CREATE TABLE IF NOT EXISTS spaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    icon VARCHAR(50),
    description VARCHAR(1000),
    is_public BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_spaces_owner_id ON spaces(owner_id);
CREATE INDEX IF NOT EXISTS idx_spaces_is_public ON spaces(is_public);

CREATE TABLE IF NOT EXISTS space_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id UUID NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    joined_at TIMESTAMP NOT NULL DEFAULT NOW(),
    invited_by UUID NOT NULL,
    UNIQUE(space_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_space_memberships_space_id ON space_memberships(space_id);
CREATE INDEX IF NOT EXISTS idx_space_memberships_user_id ON space_memberships(user_id);
