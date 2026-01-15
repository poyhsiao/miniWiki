-- ============================================
-- miniWiki Sync Service Schema
-- Version: 006
-- Created: 2026-01-14
-- Description: Add sync-related columns for CRDT document synchronization
-- ============================================

-- Add version column for optimistic locking
ALTER TABLE documents ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

-- Add last_synced_at column to track sync state
ALTER TABLE documents ADD COLUMN IF NOT EXISTS last_synced_at TIMESTAMP;

-- Add vector_clock column to store CRDT state vector as JSONB
ALTER TABLE documents ADD COLUMN IF NOT EXISTS vector_clock JSONB NOT NULL DEFAULT '{}';

-- Add client_id column for CRDT client identification
ALTER TABLE documents ADD COLUMN IF NOT EXISTS client_id UUID;

-- Add sync_state column for document sync status
ALTER TABLE documents ADD COLUMN IF NOT EXISTS sync_state VARCHAR(20) NOT NULL DEFAULT 'synced' CHECK (sync_state IN ('synced', 'pending', 'conflicting', 'offline'));

-- Create index for version-based queries
CREATE INDEX IF NOT EXISTS idx_documents_version ON documents(version);

-- Create index for sync state queries
CREATE INDEX IF NOT EXISTS idx_documents_sync_state ON documents(sync_state) WHERE sync_state != 'synced';

-- Create index for last synced queries
CREATE INDEX IF NOT EXISTS idx_documents_last_synced ON documents(last_synced_at DESC);

-- Create index for vector_clock queries
CREATE INDEX IF NOT EXISTS idx_documents_vector_clock ON documents USING GIN (vector_clock);

-- ============================================
-- SYNC OPERATIONS TABLE
-- ============================================

-- Create table for tracking sync operations (for CRDT operations)
CREATE TABLE IF NOT EXISTS sync_operations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    client_id UUID NOT NULL,
    clock BIGINT NOT NULL,
    operation_type VARCHAR(20) NOT NULL CHECK (operation_type IN ('insert', 'delete', 'update', 'move')),
    position INTEGER,
    content TEXT,
    length INTEGER,
    parent_id UUID,
    vector_clock JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(document_id, client_id, clock)
);

CREATE INDEX IF NOT EXISTS idx_sync_operations_document ON sync_operations(document_id);
CREATE INDEX IF NOT EXISTS idx_sync_operations_client ON sync_operations(document_id, client_id);
CREATE INDEX IF NOT EXISTS idx_sync_operations_clock ON sync_operations(document_id, clock);

-- ============================================
-- AWARENESS STATES TABLE
-- ============================================

-- Create table for storing WebSocket awareness states (cursor positions, selections)
CREATE TABLE IF NOT EXISTS awareness_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    client_id UUID NOT NULL,
    user_name VARCHAR(100),
    user_color VARCHAR(7),
    cursor_position INTEGER,
    selection_start INTEGER,
    selection_end INTEGER,
    last_active_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(document_id, client_id)
);

CREATE INDEX IF NOT EXISTS idx_awareness_document ON awareness_states(document_id);
CREATE INDEX IF NOT EXISTS idx_awareness_client ON awareness_states(client_id);
CREATE INDEX IF NOT EXISTS idx_awareness_active ON awareness_states(last_active_at DESC);

-- ============================================
-- SYNC METADATA TABLE
-- ============================================

-- Create table for tracking global sync metadata
CREATE TABLE IF NOT EXISTS sync_metadata (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    last_full_sync TIMESTAMP,
    last_incremental_sync TIMESTAMP,
    total_sync_operations BIGINT NOT NULL DEFAULT 0,
    total_conflicts INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Insert initial row if not exists
INSERT INTO sync_metadata (id) VALUES (1) ON CONFLICT (id) DO NOTHING;

