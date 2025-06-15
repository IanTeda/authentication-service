-- ============================================================================
-- Migration: 00000000004_create_password_reset_table.sql
-- Purpose:   Create the password_resets table for managing password reset tokens.
-- Author:    Ian Teda
-- Date:      2025-06-02
--
-- This migration creates a table to store password reset tokens, including:
--   - user_id: references the users table
--   - token: unique reset token
--   - expires_at: expiration timestamp
--   - used: whether the token has been used
-- ============================================================================

-- Create Email Verifications table
CREATE TABLE password_resets (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(512) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    is_used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Create index's for faster look ups with cursor based pagination
-- Index for faster cursor pagination look ups
CREATE INDEX idx_password_resets_created_at_id
    ON password_resets (created_at, id);

-- Index for faster look ups by user_id
CREATE INDEX idx_password_resets_user_id
    ON password_resets (user_id);