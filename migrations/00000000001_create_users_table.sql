-- ./migrations/{timestamp}_migration_action.sql
-- Create User Role Postgres Enum (Type), need to drop first as migration will crash
-- trying to create type. Need to be lower case per sqlx::type derive
DROP TYPE IF EXISTS user_role CASCADE;  -- If we don't drop first sqlx migration crashes on create type
CREATE TYPE user_role AS ENUM ('admin', 'user', 'guest');

-- Create Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID NOT NULL,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'user',
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_on TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    PRIMARY KEY (id)
);

-- Create an index's for quicker find
-- https://www.slingacademy.com/article/postgresql-how-to-set-index-on-a-table-column/
CREATE UNIQUE INDEX idx_unique_email ON users (email);

-- Password: Personal-L3dger-password!
INSERT INTO users (
    id, email, name, password_hash, role, is_active, is_verified, created_on
) VALUES (
    '019071c5-a31c-7a0e-befa-594702122e75',
    'ledger@teda.id.au',
    'Admin',
    '$argon2id$v=19$m=15000,t=2,p=1$VPNpCs5rz4G3W97dnTBU6A$slVdSjMeQIRBBzz8srnGgOobKupeOM+z9oX41QlqxZQ',
    'admin',
    'true',
    'true',
    '2019-10-17T00:00:00.000000Z'
)