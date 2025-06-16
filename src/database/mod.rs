//-- ./src/database/mod.rs

// #![allow(unused)] // For development only

//! Database module for the authentication service.
//!
//! This module provides initialisation and access to the application's database tables,
//! including connection pool setup, migrations, and re-exports of user and session models.
//!
//! # Contents
//! - Connection pool initialisation and migration runner
//! - Import user and session database models and logic
//! - Re-exports modules for convenient access in other parts of the application

use crate::{configuration::DatabaseConfiguration, prelude::*};
use sqlx::{postgres::PgPoolOptions, PgPool};

// Module imports
mod email_verification;
// mod password_reset;
mod sessions;
mod users;

// Reexport modules for cleaner code
pub use email_verification::EmailVerifications;
pub use sessions::Sessions;
pub use users::Users;

/// Initialize the PostgreSQL connection pool and run database migrations.
///
/// # Parameters
/// * `database_configuration` - Reference to the application's database configuration.
///
/// # Returns
/// * `Ok(PgPool)` - The initialize PostgreSQL connection pool if successful.
/// * `Err(AuthenticationError)` - If the connection or migration fails.
///
/// # Behaviors
/// - Builds a lazy connection pool using the provided configuration.
/// - Runs all pending SQLx migrations from the `./migrations` directory before returning the pool.
/// - Returns an error if the connection or migration fails.
pub async fn init_pool(
    database_configuration: &DatabaseConfiguration,
) -> Result<PgPool, AuthenticationError> {
    // Build connection pool
    let database =
        PgPoolOptions::new().connect_lazy_with(database_configuration.connection());

    // Migrate database
    sqlx::migrate!("./migrations").run(&database).await?;

    // Return database
    Ok(database)
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

    #![allow(unused)]

    use crate::domain::{self, UserRole};

    use sqlx::{Pool, Postgres};

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    const DEFAULT_USER_ID: uuid::Uuid =
        uuid::Uuid::from_u128(0x019071c5a31c7a0ebefa594702122e75);

    #[sqlx::test]
    async fn test_users_table_creation_migrates_correctly(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- 1. Check that the users table exists and can be queried
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&database)
            .await?;
        assert!(row.0 >= 1, "users table should have at least one row");

        //-- 2. Check that the default admin user exists and all columns are present and correct
        let admin = sqlx::query_as::<_, (
            uuid::Uuid,                      // id
            String,                          // email
            String,                          // name
            String,                          // password_hash
            UserRole,                        // role
            bool,                            // is_active
            bool,                            // is_verified
            chrono::DateTime<chrono::Utc>,   // created_on
        )>("SELECT id, email, name, password_hash, role, is_active, is_verified, created_on FROM users WHERE role = 'admin' LIMIT 1")
        .fetch_one(&database)
        .await?;

        assert_eq!(admin.0.to_string(), DEFAULT_USER_ID.to_string());
        assert_eq!(admin.1, "default_ams@teda.id.au");
        assert_eq!(admin.2, "Admin");
        assert_eq!(admin.3, "$argon2id$v=19$m=15000,t=2,p=1$HBwgCOwk9o745vPiPI/0iA$TozkH3DlprgOaWhMOU4xE1xrVGJkdUWofJujyiJ4j+U");
        assert_eq!(admin.4, UserRole::Admin);
        assert_eq!(admin.5, true);
        assert_eq!(admin.6, true);
        assert_eq!(admin.7.to_rfc3339(), "2019-10-17T00:00:00+00:00");

        //-- 3. Check that all columns exist by selecting even if it returns an empty row
        let result = sqlx::query_as::<_, (
            uuid::Uuid,                      // id
            String,                          // email
            String,                          // name
            String,                          // password_hash
            UserRole,                        // role
            bool,                            // is_active
            bool,                            // is_verified
            chrono::DateTime<chrono::Utc>,   // created_on
        )>("SELECT id, email, name, password_hash, role, is_active, is_verified, created_on FROM users LIMIT 1")
        .fetch_optional(&database)
        .await?;

        if let Some((
            id,
            email,
            name,
            password_hash,
            role,
            is_active,
            is_verified,
            created_on,
        )) = result
        {
            assert!(!id.is_nil(), "id should be a valid UUID");
            assert!(!email.is_empty(), "email should not be empty");
            assert!(!name.is_empty(), "name should not be empty");
            assert!(
                !password_hash.is_empty(),
                "password_hash should not be empty"
            );
            assert!(!role.is_empty(), "role should not be empty");
            // is_active and is_verified are bools, no further check needed
            assert!(
                created_on.timestamp() > 0,
                "created_on should be a valid timestamp"
            );
        }

        //-- 4. Check table constraint

        // a. email is UNIQUE
        let duplicate_email_result = sqlx::query(
            "INSERT INTO users (id, email, name, password_hash, role, is_active, is_verified, created_on)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(uuid::Uuid::new_v4())
        .bind("default_ams@teda.id.au") // duplicate email
        .bind("Another Admin")
        .bind("dummy_hash")
        .bind(domain::UserRole::Admin)
        .bind(true)
        .bind(true)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await;

        assert!(
            duplicate_email_result.is_err(),
            "Should not allow duplicate emails due to UNIQUE constraint"
        );

        // b. NOT NULL constraints (try inserting a row with NULL for NOT NULL columns)
        let null_email_result = sqlx::query(
            "INSERT INTO users (id, name, password_hash, role, is_active, is_verified, created_on)
            VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(uuid::Uuid::new_v4())
        // .bind(NULL) -- omitted email
        .bind("No Email")
        .bind("dummy_hash")
        .bind(domain::UserRole::User)
        .bind(true)
        .bind(true)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await;

        assert!(
            null_email_result.is_err(),
            "Should not allow NULL for NOT NULL columns"
        );

        // c. PRIMARY KEY constraint (duplicate id)
        let duplicate_id = domain::RowID::mock();
        let duplicate_id = duplicate_id.into_uuid();
        let _ = sqlx::query(
            "INSERT INTO users (id, email, name, password_hash, role, is_active, is_verified, created_on)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(duplicate_id)
        .bind("unique_email@teda.id.au")
        .bind("Unique User")
        .bind("dummy_hash")
        .bind(domain::UserRole::User)
        .bind(true)
        .bind(true)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await?;

        let duplicate_id_result = sqlx::query(
            "INSERT INTO users (id, email, name, password_hash, role, is_active, is_verified, created_on)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(duplicate_id) // duplicate id
        .bind("another_unique@teda.id.au")
        .bind("Another User")
        .bind("dummy_hash")
        .bind(domain::UserRole::User)
        .bind(true)
        .bind(true)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await;

        assert!(
            duplicate_id_result.is_err(),
            "Should not allow duplicate IDs due to PRIMARY KEY constraint"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_sessions_table_creation_migrates_correctly(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- 1. Check that the sessions table exists and can be queried
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&database)
            .await?;
        // This just checks the table exists; row count may be 0 if no sessions yet
        assert!(row.0 >= 0, "sessions table should exist");

        //-- 2. Check that all columns exist by selecting one row (if any)
        let result = sqlx::query_as::<_, (
            uuid::Uuid,                             // id
            uuid::Uuid,                             // user_id
            chrono::DateTime<chrono::Utc>,          // logged_in_at
            Option<i32>,                            // login_ip
            chrono::DateTime<chrono::Utc>,          // expires_on
            String,                                 // refresh_token
            bool,                                   // is_active
            Option<chrono::DateTime<chrono::Utc>>,  // logged_out_at
            Option<i32>,                            // logout_ip
        )>("SELECT id, user_id, logged_in_at, login_ip, expires_on, refresh_token, is_active, logged_out_at, logout_ip FROM sessions LIMIT 1")
            .fetch_optional(&database)
            .await?;

        // If there is at least one session, assert the types are as expected
        if let Some((
            id,
            user_id,
            logged_in_at,
            login_ip,
            expires_on,
            refresh_token,
            is_active,
            logged_out_at,
            logout_ip,
        )) = result
        {
            // Just basic assertions to ensure columns are present and types are correct
            assert!(
                !refresh_token.is_empty(),
                "refresh_token should not be empty"
            );
            // You can add more specific assertions if you insert a test session row in your migrations
        }

        //-- 3. Check table constraints

        // a. PRIMARY KEY constraint (duplicate id)
        let duplicate_id = domain::RowID::mock().into_uuid();
        // Insert a session with a unique id
        let _ = sqlx::query(
            "INSERT INTO sessions (id, user_id, logged_in_at, login_ip, expires_on, refresh_token, is_active, logged_out_at, logout_ip)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(duplicate_id)
        .bind(DEFAULT_USER_ID)
        .bind(chrono::Utc::now())
        .bind(Some(123456789))
        .bind(chrono::Utc::now() + chrono::Duration::days(1))
        .bind("refresh_token_value")
        .bind(true)
        .bind(None::<chrono::DateTime<chrono::Utc>>)
        .bind(None::<i32>)
        .execute(&database)
        .await?;

        // Try inserting another session with the same id
        let duplicate_id_result = sqlx::query(
            "INSERT INTO sessions (id, user_id, logged_in_at, login_ip, expires_on, refresh_token, is_active, logged_out_at, logout_ip)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(duplicate_id)
        .bind(DEFAULT_USER_ID)
        .bind(chrono::Utc::now())
        .bind(Some(987654321))
        .bind(chrono::Utc::now() + chrono::Duration::days(2))
        .bind("another_refresh_token")
        .bind(false)
        .bind(None::<chrono::DateTime<chrono::Utc>>)
        .bind(None::<i32>)
        .execute(&database)
        .await;

        assert!(
            duplicate_id_result.is_err(),
            "Should not allow duplicate IDs due to PRIMARY KEY constraint"
        );

        // b. NOT NULL constraints (try inserting a row with NULL for NOT NULL columns)
        let null_user_id_result = sqlx::query(
            "INSERT INTO sessions (id, logged_in_at, expires_on, refresh_token, is_active)
            VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(domain::RowID::mock().into_uuid())
        .bind(chrono::Utc::now())
        .bind(chrono::Utc::now() + chrono::Duration::days(1))
        .bind("refresh_token_value")
        .bind(true)
        .execute(&database)
        .await;

        assert!(
            null_user_id_result.is_err(),
            "Should not allow NULL for NOT NULL columns (user_id)"
        );

        let null_refresh_token_result = sqlx::query(
            "INSERT INTO sessions (id, user_id, logged_in_at, expires_on, is_active)
            VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(domain::RowID::mock().into_uuid())
        .bind(DEFAULT_USER_ID)
        .bind(chrono::Utc::now())
        .bind(chrono::Utc::now() + chrono::Duration::days(1))
        .bind(true)
        .execute(&database)
        .await;

        assert!(
            null_refresh_token_result.is_err(),
            "Should not allow NULL for NOT NULL columns (refresh_token)"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_password_resets_table_creation_migrates_correctly(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- 1. Check that the password_resets table exists and can be queried
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM password_resets")
            .fetch_one(&database)
            .await?;
        // Table should exist; row count may be 0 if no resets yet
        assert!(row.0 >= 0, "password_resets table should exist");

        // 2. Check that all columns exist by selecting one row (if any)
        let result = sqlx::query_as::<_, (
            uuid::Uuid,                      // id
            uuid::Uuid,                      // user_id
            String,                          // token
            chrono::DateTime<chrono::Utc>,   // expires_at
            bool,                            // used
            chrono::DateTime<chrono::Utc>,   // created_at
        )>("SELECT id, user_id, token, expires_at, used, created_at FROM password_resets LIMIT 1")
            .fetch_optional(&database)
            .await?;

        // If there is at least one password reset row, assert the types and values
        if let Some((id, user_id, token, expires_at, used, created_at)) = result {
            assert!(!id.is_nil(), "id should be a valid UUID");
            assert!(!user_id.is_nil(), "user_id should be a valid UUID");
            assert!(!token.is_empty(), "token should not be empty");
            assert!(
                expires_at.timestamp() > 0,
                "expires_at should be a valid timestamp"
            );
            // used is a bool, no further check needed
            assert!(
                created_at.timestamp() > 0,
                "created_at should be a valid timestamp"
            );
        }

        //-- 3. Check table constraints

        // a. UNIQUE constraint on token
        let unique_token = format!("token-{}", uuid::Uuid::new_v4());
        let _ = sqlx::query(
            "INSERT INTO password_resets (id, user_id, token, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(domain::RowID::mock().into_uuid())
        .bind(DEFAULT_USER_ID)
        .bind(&unique_token)
        .bind(chrono::Utc::now() + chrono::Duration::days(1))
        .bind(false)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await?;

        let duplicate_token_result = sqlx::query(
            "INSERT INTO password_resets (id, user_id, token, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(domain::RowID::mock().into_uuid())
        .bind(DEFAULT_USER_ID)
        .bind(&unique_token) // duplicate token
        .bind(chrono::Utc::now() + chrono::Duration::days(2))
        .bind(false)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await;

        assert!(
            duplicate_token_result.is_err(),
            "Should not allow duplicate tokens due to UNIQUE constraint"
        );

        // b. NOT NULL constraints (try inserting a row with NULL for NOT NULL columns)
        let null_token_result = sqlx::query(
            "INSERT INTO password_resets (id, user_id, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(domain::RowID::mock().into_uuid())
        .bind(DEFAULT_USER_ID)
        // .bind(NULL) -- omitted token
        .bind(chrono::Utc::now() + chrono::Duration::days(1))
        .bind(false)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await;

        assert!(
            null_token_result.is_err(),
            "Should not allow NULL for NOT NULL columns (token)"
        );

        // c. PRIMARY KEY constraint (duplicate id)
        let duplicate_id = domain::RowID::mock().into_uuid();
        let _ = sqlx::query(
            "INSERT INTO password_resets (id, user_id, token, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(duplicate_id)
        .bind(DEFAULT_USER_ID)
        .bind(format!("token-{}", domain::RowID::mock().into_uuid()))
        .bind(chrono::Utc::now() + chrono::Duration::days(1))
        .bind(false)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await?;

        let duplicate_id_result = sqlx::query(
            "INSERT INTO password_resets (id, user_id, token, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(duplicate_id) // duplicate id
        .bind(DEFAULT_USER_ID)
        .bind(format!("token-{}", uuid::Uuid::new_v4()))
        .bind(chrono::Utc::now() + chrono::Duration::days(2))
        .bind(false)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await;

        assert!(
            duplicate_id_result.is_err(),
            "Should not allow duplicate IDs due to PRIMARY KEY constraint"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_email_verifications_table_creation_migrates_correctly(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- 1. Check that the email_verifications table exists and can be queried
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM email_verifications")
            .fetch_one(&database)
            .await?;
        // Table should exist; row count may be 0 if no verifications yet
        assert!(row.0 >= 0, "email_verifications table should exist");

        //-- 2. Check that all columns exist by selecting one row (if any)
        let result = sqlx::query_as::<_, (
            uuid::Uuid,                      // id
            uuid::Uuid,                      // user_id
            String,                          // token
            chrono::DateTime<chrono::Utc>,   // expires_at
            bool,                            // used
            chrono::DateTime<chrono::Utc>,   // created_at
        )>("SELECT id, user_id, token, expires_at, used, created_at FROM email_verifications LIMIT 1")
        .fetch_optional(&database)
        .await?;

        // If there is at least one email verification row, assert the types and values
        if let Some((id, user_id, token, expires_at, used, created_at)) = result {
            assert!(!id.is_nil(), "id should be a valid UUID");
            assert!(!user_id.is_nil(), "user_id should be a valid UUID");
            assert!(!token.is_empty(), "token should not be empty");
            assert!(
                expires_at.timestamp() > 0,
                "expires_at should be a valid timestamp"
            );
            // used is a bool, no further check needed
            assert!(
                created_at.timestamp() > 0,
                "created_at should be a valid timestamp"
            );
        }

        //-- 3. Check table constraints
        // a. UNIQUE constraint on token
        let unique_token = format!("token-{}", uuid::Uuid::new_v4());
        let _ = sqlx::query(
            "INSERT INTO email_verifications (id, user_id, token, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(domain::RowID::mock().into_uuid())
        .bind(DEFAULT_USER_ID)
        .bind(&unique_token)
        .bind(chrono::Utc::now() + chrono::Duration::days(1))
        .bind(false)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await?;

        let duplicate_token_result = sqlx::query(
            "INSERT INTO email_verifications (id, user_id, token, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(domain::RowID::mock().into_uuid())
        .bind(DEFAULT_USER_ID)
        .bind(&unique_token) // duplicate token
        .bind(chrono::Utc::now() + chrono::Duration::days(2))
        .bind(false)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await;

        assert!(
            duplicate_token_result.is_err(),
            "Should not allow duplicate tokens due to UNIQUE constraint"
        );

        // b. NOT NULL constraints (try inserting a row with NULL for NOT NULL columns)
        let null_token_result = sqlx::query(
            "INSERT INTO email_verifications (id, user_id, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(domain::RowID::mock().into_uuid())
        .bind(DEFAULT_USER_ID)
        // .bind(NULL) -- omitted token
        .bind(chrono::Utc::now() + chrono::Duration::days(1))
        .bind(false)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await;

        assert!(
            null_token_result.is_err(),
            "Should not allow NULL for NOT NULL columns (token)"
        );

        // c. PRIMARY KEY constraint (duplicate id)
        let duplicate_id = domain::RowID::mock().into_uuid();
        let _ = sqlx::query(
            "INSERT INTO email_verifications (id, user_id, token, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(duplicate_id)
        .bind(DEFAULT_USER_ID)
        .bind(format!("token-{}", domain::RowID::mock().into_uuid()))
        .bind(chrono::Utc::now() + chrono::Duration::days(1))
        .bind(false)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await?;

        let duplicate_id_result = sqlx::query(
            "INSERT INTO email_verifications (id, user_id, token, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(duplicate_id) // duplicate id
        .bind(DEFAULT_USER_ID)
        .bind(format!("token-{}", uuid::Uuid::new_v4()))
        .bind(chrono::Utc::now() + chrono::Duration::days(2))
        .bind(false)
        .bind(chrono::Utc::now())
        .execute(&database)
        .await;

        assert!(
            duplicate_id_result.is_err(),
            "Should not allow duplicate IDs due to PRIMARY KEY constraint"
        );

        Ok(())
    }
}
