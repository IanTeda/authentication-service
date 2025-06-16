-- ============================================================================
-- Migration: 00000000003_create_email_verification_table.sql
-- Purpose:   Create the email_verifications table for managing email verification tokens.
-- Author:    Ian Teda
-- Date:      2025-06-02
--
-- This migration creates a table to store email verification tokens used for
-- confirming user email addresses during registration or email change processes.
-- The table supports token lifecycle management including expiration and usage tracking.
--
-- Dependencies:
--   - Requires users table (created in migration 00000000001)
--   - Uses UUID v7 for time-ordered primary keys
--   - Supports timezone-aware timestamps
--
-- Security Considerations:
--   - Tokens are cryptographically secure wih JWT
--   - Expired tokens are periodically cleaned up
--   - Used tokens remain in table for audit purposes
--
-- Performance Notes:
--   - Includes optimised indexes for common query patterns
--   - Supports efficient cursor-based pagination
--   - Fast look ups by user_id for user-specific operations
-- ============================================================================

-- Create Email Verifications table
-- This table stores email verification tokens with full lifecycle tracking
CREATE TABLE email_verifications (
    -- Primary key using UUID v7 for time-ordered unique identification
    -- UUID v7 provides chronological ordering which aids in pagination and debugging
    id UUID PRIMARY KEY NOT NULL,
    
    -- Foreign key reference to the users table
    -- CASCADE DELETE ensures cleanup when users are deleted
    -- NOT NULL ensures every verification is tied to a valid user
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- The verification token VARCHAR(512) accommodates JWT tokens and 
    -- other token formats.
    -- UNIQUE constraint prevents token reuse and ensures security
    token VARCHAR(512) NOT NULL UNIQUE,
    
    -- Expiration timestamp for the verification token
    -- TIMESTAMPTZ ensures timezone awareness for global applications
    -- NOT NULL ensures all tokens have defined expiration
    -- Expired tokens should be considered invalid but kept for audit
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Boolean flag indicating whether the token has been used
    -- DEFAULT FALSE ensures new tokens are initially unused
    -- Once set to TRUE, token should not be reusable
    -- Used tokens are kept for audit trail and analytics
    is_used BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Timestamp when the verification record was created
    -- TIMESTAMPTZ with DEFAULT now() for automatic timestamp creation
    -- Immutable field used for audit trails and analytics
    -- NOT NULL ensures all records have creation timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- Timestamp when the verification record was last updated
    -- NULL by default, set when token status changes (e.g., marked as used)
    -- Used for tracking token lifecycle and audit purposes
    -- Optional field for update tracking
    updated_at TIMESTAMPTZ
);

-- Performance Optimisation Indexes
-- These indexes support common query patterns and improve application performance

-- Composite index for efficient cursor-based pagination
-- Supports ORDER BY created_at, id queries with optimal performance
-- Essential for paginated lists of verifications sorted by creation time
-- The id component ensures deterministic ordering for records with identical timestamps
CREATE INDEX idx_email_verifications_created_at_id
    ON email_verifications (created_at, id);

-- Index for fast user-specific look ups
CREATE INDEX idx_email_verifications_user_id
    ON email_verifications (user_id);

-- Index for token expiration cleanup jobs:
CREATE INDEX idx_email_verifications_expires_at 
    ON email_verifications (expires_at) WHERE is_used = FALSE;

-- Index for finding unused, non-expired tokens:
CREATE INDEX idx_email_verifications_active 
    ON email_verifications (user_id, expires_at) WHERE is_used = FALSE;

-- Partial index for cleanup of expired tokens:
-- This query will use the expires_at index efficiently
-- DELETE FROM email_verifications 
-- WHERE expires_at < now() - interval '30 days';
CREATE INDEX idx_email_verifications_expires_at_cleanup
    ON email_verifications (expires_at);
