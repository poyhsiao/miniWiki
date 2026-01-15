-- ============================================
-- miniWiki Full-Text Search Enhancement
-- Version: 007
-- Created: 2026-01-15
-- Description: Enable pg_trgm extension and create GIN indexes for full-text search
-- ============================================

-- Enable pg_trgm extension for trigram similarity search
-- This enables ILIKE and similarity functions to work much faster
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ============================================
-- FULL-TEXT SEARCH INDEXES
-- ============================================

-- Create GIN index on documents for full-text search
-- This dramatically improves search performance for both ILIKE and full-text queries
CREATE INDEX IF NOT EXISTS idx_documents_search_title 
ON documents USING gin (title gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_documents_search_content 
ON documents USING gin (content_text gin_trgm_ops);

-- Composite index for combined title + content search
CREATE INDEX IF NOT EXISTS idx_documents_search_combined 
ON documents USING gin (to_tsvector('english', COALESCE(title, '') || ' ' || COALESCE(content_text, '')));

-- Create GIN index on spaces for search
CREATE INDEX IF NOT EXISTS idx_spaces_search_name 
ON spaces USING gin (name gin_trgm_ops);

-- Create GIN index on users for search
CREATE INDEX IF NOT EXISTS idx_users_search_email 
ON users USING gin (email gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_users_search_name 
ON users USING gin (display_name gin_trgm_ops);

-- Create GIN index on comments for search
CREATE INDEX IF NOT EXISTS idx_comments_search 
ON comments USING gin (content gin_trgm_ops);

-- ============================================
-- SEARCH PERFORMANCE VIEWS
-- ============================================

-- Create a materialized view for search statistics (optional, for large datasets)
-- This can be refreshed periodically for better search performance
CREATE MATERIALIZED VIEW IF NOT EXISTS search_document_stats AS
SELECT 
    d.id as document_id,
    d.space_id,
    d.title,
    d.content_text,
    COALESCE(d.updated_at, d.created_at) as last_activity,
    s.name as space_name,
    ts_rank(
        setweight(to_tsvector('english', COALESCE(d.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(d.content_text, '')), 'B'),
        to_tsquery('english', regexp_replace('rust|flutter|dart|programming', '\s+', ' & ', 'g'))
    ) as search_rank
FROM documents d
JOIN spaces s ON d.space_id = s.id
WHERE d.is_archived = false;

-- Create index on the materialized view
CREATE INDEX IF NOT EXISTS idx_search_doc_stats_rank 
ON search_document_stats (search_rank DESC);

-- ============================================
-- FUNCTION: Optimized search function
-- ============================================

-- Create a function to perform ranked full-text search
-- This provides better ranking than simple ILIKE queries
CREATE OR REPLACE FUNCTION search_documents_ranked(
    p_user_id UUID,
    p_query TEXT,
    p_space_id UUID DEFAULT NULL,
    p_limit INTEGER DEFAULT 20,
    p_offset INTEGER DEFAULT 0
)
RETURNS TABLE (
    document_id UUID,
    space_id UUID,
    space_name TEXT,
    title TEXT,
    snippet TEXT,
    score REAL,
    total_count BIGINT
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_query_tsvector tsvector;
    v_query_pattern TEXT;
BEGIN
    -- Convert user query to tsvector for full-text search
    v_query_tsvector := to_tsvector('english', p_query);
    
    -- Also create a pattern for ILIKE fallback
    v_query_pattern := '%' || p_query || '%';

    -- Return results with ranking
    RETURN QUERY
    WITH ranked_results AS (
        SELECT 
            d.id as document_id,
            d.space_id,
            s.name as space_name,
            d.title,
            d.content_text,
            -- Full-text search rank using ts_rank
            COALESCE(
                ts_rank(
                    setweight(to_tsvector('english', COALESCE(d.title, '')), 'A') ||
                    setweight(to_tsvector('english', COALESCE(d.content_text, '')), 'B'),
                    v_query_tsvector
                ) * 2.0 +
                -- Boost title matches
                CASE 
                    WHEN d.title ILIKE v_query_pattern THEN 1.5 
                    ELSE 0.0 
                END +
                -- Boost recent updates
                CASE 
                    WHEN d.updated_at > NOW() - INTERVAL '7 days' THEN 0.5
                    WHEN d.updated_at > NOW() - INTERVAL '30 days' THEN 0.2
                    ELSE 0.0
                END,
                0.0
            ) as score,
            d.content as content
        FROM documents d
        JOIN spaces s ON d.space_id = s.id
        WHERE d.is_archived = false
        AND (
            -- Full-text search match
            setweight(to_tsvector('english', COALESCE(d.title, '')), 'A') ||
            setweight(to_tsvector('english', COALESCE(d.content_text, '')), 'B')
            @@ v_query_tsvector
            -- Fallback to ILIKE for simple substring matches
            OR d.title ILIKE v_query_pattern 
            OR d.content_text ILIKE v_query_pattern
        )
        AND EXISTS (
            SELECT 1 FROM space_memberships sm
            WHERE sm.space_id = d.space_id
            AND sm.user_id = p_user_id
        )
        AND (p_space_id IS NULL OR d.space_id = p_space_id)
    )
    SELECT 
        rr.document_id,
        rr.space_id,
        rr.space_name,
        rr.title,
        LEFT(COALESCE(rr.content_text, ''), 150) as snippet,
        rr.score,
        COUNT(*) OVER() as total_count
    FROM ranked_results rr
    ORDER BY rr.score DESC, rr.content_text ASC
    LIMIT p_limit OFFSET p_offset;
END;
$$;

-- ============================================
-- MIGRATION COMPLETE
-- ============================================
-- Run this migration after the initial schema
-- Migration is idempotent - safe to run multiple times
