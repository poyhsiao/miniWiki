-- Add author_name and author_avatar columns to comments table
-- Migration: 011_comments_add_author_fields.sql
-- Date: 2026-01-15

-- Add optional author_name and author_avatar columns for denormalized display
ALTER TABLE comments ADD COLUMN IF NOT EXISTS author_name VARCHAR(255);
ALTER TABLE comments ADD COLUMN IF NOT EXISTS author_avatar TEXT;

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_comments_author_name ON comments(author_name);

-- Backfill existing comments with placeholder values
UPDATE comments SET author_name = 'Unknown User' WHERE author_name IS NULL;
