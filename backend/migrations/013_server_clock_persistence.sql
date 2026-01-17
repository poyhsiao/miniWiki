-- ============================================
-- miniWiki Sync Service - Server Clock Persistence
-- Version: 013
-- Created: 2026-01-17
-- Description: Add server_clock column to sync_metadata for persistent clock state
-- ============================================

-- Add server_clock column to sync_metadata table
ALTER TABLE sync_metadata ADD COLUMN IF NOT EXISTS server_clock BIGINT NOT NULL DEFAULT 0;
