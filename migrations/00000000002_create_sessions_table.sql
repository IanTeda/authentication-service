-- ./migrations/{timestamp}_migration_action.sql

-- # Create Sessions table
-- 
-- This table will store the user sessions. Keeping track of the user's login 
-- and logout times, IP addresses, session status (is_active) and the refresh token.
CREATE TABLE IF NOT EXISTS sessions (
    id UUID NOT NULL,
    user_id UUID NOT NULL,
    logged_in_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    login_ip INT,
    expires_on TIMESTAMP WITH TIME ZONE NOT NULL,
    refresh_token TEXT NOT NULL,
    is_active BOOLEAN DEFAULT false NOT NULL,
    logged_out_at TIMESTAMP WITH TIME ZONE,
    logout_ip INT,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create an index for quicker lookup by created_on and id with cursor based pagination
CREATE INDEX idx_sessions_logged_in_at_id ON sessions (logged_in_at, id);

-- Indexes for common queries

-- Index for looking up user's sessions
CREATE INDEX idx_sessions_user_id ON sessions (user_id);

-- Index for finding active sessions
CREATE INDEX idx_sessions_is_active ON sessions (is_active) WHERE is_active = true;