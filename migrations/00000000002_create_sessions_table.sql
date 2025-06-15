-- ============================================================================
-- Migration: 00000000002_create_sessions_table.sql
-- Purpose:   Create the sessions table for tracking user sessions.
-- Author:    Ian Teda
-- Date:      2025-06-02
--
-- This migration:
--   - Creates the sessions table to store user session data (login/logout times, IPs, status, refresh token)
--   - Adds indexes for efficient look ups by login time, user, and active status
-- ============================================================================

-- Create Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    logged_in_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    login_ip INT,
    expires_on TIMESTAMPTZ NOT NULL,
    refresh_token VARCHAR(512) NOT NULL,
    is_active BOOLEAN DEFAULT false NOT NULL,
    logged_out_at TIMESTAMPTZ,
    logout_ip INT
);

-- Create index's for faster look ups with cursor based pagination
-- Index for looking up sessions by login time and id
CREATE INDEX idx_sessions_logged_in_at_id ON sessions (logged_in_at, id);
-- Index for looking up user's sessions
CREATE INDEX idx_sessions_user_id ON sessions (user_id);
-- Index for finding active sessions
CREATE INDEX idx_sessions_is_active ON sessions (is_active)
  WHERE is_active = true;