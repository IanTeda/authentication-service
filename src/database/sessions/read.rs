//-- ./src/database/sessions/read.rs

// #![allow(unused)] // For development only

//! Database read/query operations for the `Sessions` table.
//!
//! This module provides asynchronous functions for retrieving session records from the database,
//! including fetching by session ID, refresh token, user ID, and paginated queries for all sessions.
//!
//! All functions return a `Result` with either the requested session data or an `AuthenticationError`.
//! Errors are returned if the database query fails or if no matching session is found.
//!
//! # Examples
//!
//! ```rust
//! use crate::database::Sessions;
//! use uuid::Uuid;
//! use sqlx::Pool;
//!
//! // Fetch a session by ID
//! let session = Sessions::from_id(&Uuid::new_v7(), &database_pool).await;
//! ```

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::database::Sessions;
use crate::prelude::*;

impl Sessions {
    /// Retrieves a Sessions instance from the database by querying with the provided session UUID.
    ///
    /// # Parameters
    ///
    /// * `id` - The UUID of the session to retrieve.
    /// * `database` - The sqlx database pool to execute the query against.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Sessions` instance on success, or an `AuthenticationError` on failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails or if no session is found for the given UUID.
    #[tracing::instrument(
        name = "Get a Sessions from the database: ",
        skip(database),
        fields(
            session_id = ?id,
        )
    )]
    pub async fn from_id(
        id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<Sessions, AuthenticationError> {
        let database_record = sqlx::query_as!(
            Sessions,
            r#"
                SELECT id, user_id, logged_in_at, login_ip, expires_on, refresh_token, is_active, logged_out_at, logout_ip
                FROM sessions
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(database)
        .await?;

        tracing::debug!("Sessions database records retrieved: {database_record:#?}");

        Ok(database_record)
    }

    /// Retrieves a Sessions instance from the database by querying with the provided refresh token.
    ///
    /// # Parameters
    ///
    /// * `refresh_token` - A string slice representing the refresh token of the session to retrieve.
    /// * `database` - The sqlx database pool to execute the query against.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Sessions` instance on success, or an `AuthenticationError` on failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails or if no session is found for the given refresh token.
    #[tracing::instrument(
        name = "Get the session associated with: ",
        skip(database),
        fields(
            refresh_token = ?refresh_token,
        )
    )]
    pub async fn from_token(
        refresh_token: &str,
        database: &Pool<Postgres>,
    ) -> Result<Sessions, AuthenticationError> {
        let database_record = sqlx::query_as!(
            Sessions,
            r#"
                SELECT id, user_id, logged_in_at, login_ip, expires_on, refresh_token, is_active, logged_out_at, logout_ip
                FROM sessions
                WHERE refresh_token = $1
            "#,
            refresh_token
        )
        .fetch_one(database)
        .await?;

        tracing::debug!("Sessions database records retrieved: {database_record:#?}");

        Ok(database_record)
    }

    /// Retrieves a paginated list of Sessions for a specific user from the database.
    ///
    /// # Parameters
    ///
    /// * `user_id` - The UUID of the user whose sessions are to be retrieved.
    /// * `limit` - An i64 specifying the maximum number of Sessions to return.
    /// * `offset` - An i64 specifying the number of Sessions to skip before starting to collect the result set.
    /// * `database` - The sqlx database pool to execute the query against.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of `Sessions` on success, or an `AuthenticationError` on failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[tracing::instrument(
        name = "Get all Sessions from the database for a users id (uuid): ",
        skip(database),
        fields(
            user_id = ?user_id,
            limit = ?limit,
            offset = ?offset,
        )
    )]
    pub async fn index_from_user_id(
        user_id: &Uuid,
        limit: &usize,
        offset: &usize,
        database: &Pool<Postgres>,
    ) -> Result<Vec<Sessions>, AuthenticationError> {
        let database_records = sqlx::query_as!(
            Sessions,
            r#"
                SELECT id, user_id, logged_in_at, login_ip, expires_on, refresh_token, is_active, logged_out_at, logout_ip
                FROM sessions
                WHERE user_id = $1
                ORDER BY id
                LIMIT $2 OFFSET $3
            "#,
            user_id,
            *limit as i64,
            *offset as i64,
        )
        .fetch_all(database)
        .await?;

        tracing::debug!(
            "Sessions database records retrieved: {database_records:#?}"
        );

        Ok(database_records)
    }

    /// Retrieves a paginated list of all Sessions from the database.
    ///
    /// # Parameters
    ///
    /// * `limit` - An i64 specifying the maximum number of Sessions to return.
    /// * `offset` - An i64 specifying the number of Sessions to skip before starting to collect the result set.
    /// * `database` - The sqlx database pool to execute the query against.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of `Sessions` on success, or an `AuthenticationError` on failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[tracing::instrument(
        name = "Index of Sessions with offset and limit: ",
        skip(database),
        fields(
            limit = ?limit,
            offset = ?offset,
        )
    )]
    pub async fn index(
        limit: &usize,
        offset: &usize,
        database: &Pool<Postgres>,
    ) -> Result<Vec<Sessions>, AuthenticationError> {
        let database_records = sqlx::query_as!(
            Sessions,
            r#"
                SELECT id, user_id, logged_in_at, login_ip, expires_on, refresh_token, is_active, logged_out_at, logout_ip
                FROM sessions
                ORDER BY id
                LIMIT $1 OFFSET $2
            "#,
            *limit as i64,
            *offset as i64,
        )
        .fetch_all(database)
        .await?;

        tracing::debug!(
            "Sessions database records retrieved: {database_records:#?}"
        );

        Ok(database_records)
    }
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {
    use fake::Fake;
    use sqlx::{Pool, Postgres};

    use crate::database;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    #[sqlx::test]
    async fn get_instance_by_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        let random_user = random_user.insert(&database).await?;

        // Generate a session
        let session = database::Sessions::mock_data(&random_user).await?;

        // Insert session into database for reading later
        let session = session.insert(&database).await?;

        //-- Execute Function (Act)
        // Insert user into database
        let database_record =
            database::Sessions::from_id(&session.id, &database).await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        assert_eq!(database_record, session);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn session_for_refresh_token(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        // Generate a session
        let session = database::Sessions::mock_data(&random_user).await?;

        // Insert session into database for reading later
        let session = session.insert(&database).await?;

        //-- Execute Function (Act)
        // Insert user into database
        let database_record = database::Sessions::from_token(
            &session.refresh_token.to_string(),
            &database,
        )
        .await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        assert_eq!(database_record, session);

        // -- Return
        Ok(())
    }

    // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn count_index_from_user_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate session
            let session = database::Sessions::mock_data(&random_user).await?;

            // Insert session into the database for deleting
            let _session = session.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Get a random limit from the count
        let random_limit = (1..random_count).fake::<i64>();

        // Get a random offset from the count
        let random_offset = (1..random_count).fake::<i64>();

        // Insert user into database
        let random_limit_usize = random_limit as usize;
        let random_offset_usize = random_offset as usize;
        let database_records = database::Sessions::index_from_user_id(
            &random_user.id,
            &random_limit_usize,
            &random_offset_usize,
            &database,
        )
        .await?;
        // println!("{rows_affected:#?}");

        //-- Checks (Assertions)
        let count_less_offset: i64 = random_count - random_offset;
        let expected_records = if count_less_offset < random_limit {
            count_less_offset
        } else {
            random_limit
        };

        assert_eq!(database_records.len() as i64, expected_records);

        // -- Return
        Ok(())
    }

    // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn count_index(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate sessions
            let session = database::Sessions::mock_data(&random_user).await?;

            // Insert session into the database for deleting
            let _session = session.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Get a random limit from the count
        let random_limit = (1..random_count).fake::<i64>();

        // Get a random offset from the count
        let random_offset = (1..random_count).fake::<i64>();

        // Insert user into database
        let random_limit_usize = random_limit as usize;
        let random_offset_usize = random_offset as usize;
        let database_records = database::Sessions::index(
            &random_limit_usize,
            &random_offset_usize,
            &database,
        )
        .await?;
        // println!("{rows_affected:#?}");

        //-- Checks (Assertions)
        let count_less_offset: i64 = random_count - random_offset;
        let expected_records = if count_less_offset < random_limit {
            count_less_offset
        } else {
            random_limit
        };
        assert_eq!(database_records.len() as i64, expected_records);

        // -- Return
        Ok(())
    }

    // ...existing code...

    #[sqlx::test]
    async fn from_id_returns_error_for_nonexistent_session(
        database: Pool<Postgres>,
    ) -> Result<()> {
        use uuid::Uuid;
        let fake_id = Uuid::new_v4();
        let result = crate::database::Sessions::from_id(&fake_id, &database).await;
        assert!(result.is_err());
        Ok(())
    }

    #[sqlx::test]
    async fn from_token_returns_error_for_nonexistent_token(
        database: Pool<Postgres>,
    ) -> Result<()> {
        let fake_token = "nonexistent_token";
        let result =
            crate::database::Sessions::from_token(fake_token, &database).await;
        assert!(result.is_err());
        Ok(())
    }

    #[sqlx::test]
    async fn index_from_user_id_empty_for_no_sessions(
        database: Pool<Postgres>,
    ) -> Result<()> {
        use uuid::Uuid;
        let fake_user_id = Uuid::new_v4();
        let limit = 10;
        let offset = 0;
        let limit_usize = limit as usize;
        let offset_usize = offset as usize;
        let records = crate::database::Sessions::index_from_user_id(
            &fake_user_id,
            &limit_usize,
            &offset_usize,
            &database,
        )
        .await?;
        assert!(records.is_empty());
        Ok(())
    }

    #[sqlx::test]
    async fn index_empty_when_no_sessions(database: Pool<Postgres>) -> Result<()> {
        let limit = 10;
        let offset = 0;
        let records = crate::database::Sessions::index(
            &limit,
            &offset,
            &database,
        )
        .await?;
        assert!(records.is_empty());
        Ok(())
    }
}
