-- ./migrations/{timestamp}_migration_action.sql
-- Create Refresh Token table
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id UUID NOT NULL,
    user_id UUID NOT NULL,
    token TEXT NOT NULL,
    is_active BOOLEAN NOT NULL,
    created_on TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
