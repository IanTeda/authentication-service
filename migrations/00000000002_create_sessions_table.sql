-- ./migrations/{timestamp}_migration_action.sql

-- # Create Sessions table
-- 
-- This table will store the user sessions. Keeping track of the user's login 
-- and logout times, IP addresses, session status (is_active) and the refresh token.
CREATE TABLE IF NOT EXISTS sessions (
    id UUID NOT NULL,
    user_id UUID NOT NULL,
    login_on TIMESTAMP WITH TIME ZONE NOT NULL,
    login_ip INT,
    expires_on TIMESTAMP WITH TIME ZONE NOT NULL,
    refresh_token TEXT NOT NULL,
    is_active BOOLEAN NOT NULL,
    logout_on TIMESTAMP WITH TIME ZONE,
    logout_ip INT,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
