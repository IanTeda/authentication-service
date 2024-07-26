-- ./migrations/{timestamp}_migration_action.sql
-- Create Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID NOT NULL,
    user_id UUID NOT NULL,
    refresh_token TEXT NOT NULL,
    is_active BOOLEAN NOT NULL,
    created_on TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
