-- ./migrations/00000000003_create_logins_table.sql
-- Create Logins table
CREATE TABLE IF NOT EXISTS logins (
    id UUID NOT NULL,
    user_id UUID NOT NULL,
    login_on TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
