-- ============================================================================
-- Migration: 00000000003_create_email_verification_table.sql
-- Purpose:   Create the email_verifications table for managing email verification tokens.
-- Author:    Ian Teda
-- Date:      2025-06-02
--
-- This migration creates a table to store email verification tokens, including:
--   - user_id: references the users table
--   - token: unique verification token
--   - expires_at: expiration timestamp
--   - used: whether the token has been used
-- ============================================================================

-- Create Email Verifications table
CREATE TABLE email_verifications (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(512) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    is_used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Create index's for faster look ups with cursor based pagination
-- Index for faster cursor pagination look ups
CREATE INDEX idx_email_verifications_created_at_id
    ON email_verifications (created_at, id);
-- Index for faster look ups by user_id
CREATE INDEX idx_email_verifications_user_id
    ON email_verifications (user_id);