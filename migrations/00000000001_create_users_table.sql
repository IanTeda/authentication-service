-- ============================================================================
-- Migration: 00000000001_create_users_table.sql
-- Purpose:   Create the users table and user_role enum for the authentication service.
-- Author:    Ian Teda
-- Date:      2025-06-02
--
-- This migration:
--   - Drops and recreates the user_role enum type (admin, user, guest)
--   - Creates the users table with fields for authentication and profile
--   - Adds an index for efficient cursor-based pagination by created_on and id
--   - Inserts a default admin user (password should be changed after setup)
-- ============================================================================

-- Create User Role Postgres Enum (Type), need to drop first as migration will crash
-- trying to create type. Need to be lower case per sqlx::type derive
DROP TYPE IF EXISTS user_role CASCADE;
-- If we don't drop first sqlx migration crashes on create type
CREATE TYPE user_role AS ENUM ('admin', 'user', 'guest');

-- Create Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY NOT NULL,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash VARCHAR(512) NOT NULL,
    role user_role NOT NULL DEFAULT 'user',
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_on TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Create index's for faster look ups with cursor based pagination
-- Create an index for quicker lookup by created_on and id
CREATE INDEX idx_users_created_on_id ON users (created_on, id);
-- Create an index for quicker lookup by email
CREATE INDEX idx_users_email ON users (email);
-- Create an index for quicker lookup by role
CREATE INDEX idx_users_role ON users (role);
-- Create an index for quicker lookup by is_active
CREATE INDEX idx_users_is_active ON users (is_active);

-- Start with an admin user and password, that one should change
-- Password: "S3cret-Admin-Pas$word!"
INSERT INTO users (
        id,
        email,
        name,
        password_hash,
        role,
        is_active,
        is_verified,
        created_on
    )
VALUES (
        '019071c5-a31c-7a0e-befa-594702122e75',
        'default_ams@teda.id.au',
        'Admin',
        '$argon2id$v=19$m=15000,t=2,p=1$HBwgCOwk9o745vPiPI/0iA$TozkH3DlprgOaWhMOU4xE1xrVGJkdUWofJujyiJ4j+U',
        'admin',
        'true',
        'true',
        '2019-10-17T00:00:00.000000Z'
    );