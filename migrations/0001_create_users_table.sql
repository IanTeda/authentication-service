-- ./migrations/{timestamp}_migration_action.sql
-- Create Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID NOT NULL PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL,
    is_active BOOLEAN NOT NULL,
    created_on TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
);

-- Create an index's for quicker find
-- https://www.slingacademy.com/article/postgresql-how-to-set-index-on-a-table-column/
-- CREATE INDEX index_name ON table_name (column_name);
CREATE UNIQUE INDEX idx_unique_email ON users (email);