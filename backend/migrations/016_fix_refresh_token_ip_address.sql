-- Migration: 016_fix_refresh_token_ip_address
-- Fix: Change ip_address from INET to VARCHAR to allow NULL text values

ALTER TABLE refresh_tokens ALTER COLUMN ip_address TYPE VARCHAR(45);
