//-- ./src/database/sessions/delete.rs

// #![allow(unused)] // For development only

//! Session deletion logic for the authentication service.
//!
//! This module provides functions to delete session records from the database,
//! including deletion by session instance, session ID, user ID, bulk user and bulk deletion.
//!
//! # Contents
//! - Delete a single session by instance or ID
//! - Delete all sessions for a specific user
//! - Delete all sessions in the database
//! - Unit tests for all deletion scenarios

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::database::Sessions;
use crate::prelude::*;

// Implied for the sessions model in `./model.rs`
impl Sessions {
    //// Delete this session from the database.
    ///
    /// Executes a SQL `DELETE` statement to remove the session record identified by its `id`.
    ///
    /// # Parameters
    /// * `self` - The `Sessions` instance to be deleted.
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of rows deleted (should be 1 if the session existed, 0 otherwise).
    /// * `Err(AuthenticationError)` - If the database operation fails.
    ///
    /// # Tracing
    /// - Adds the session `id` to the tracing span for observability.
    #[tracing::instrument(
        name = "Delete Sessions instance from the database: ",
        skip(database),
        fields(
            id = ?self.id
        )
    )]
    pub async fn delete(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                DELETE
                FROM sessions
                WHERE id = $1
            "#,
            self.id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions rows database records deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Delete a session from the database by its unique session ID.
    ///
    /// Executes a SQL `DELETE` statement to remove the session record identified by the provided session `id`.
    ///
    /// # Parameters
    /// * `id` - The UUID of the session to be deleted.
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of rows deleted (should be 1 if the session existed, 0 otherwise).
    /// * `Err(AuthenticationError)` - If the database operation fails.
    ///
    /// # Tracing
    /// - Adds the session `id` to the tracing span for observability.
    #[tracing::instrument(
        name = "Delete Sessions from the database: ",
        skip(database),
        fields(
            id = ?id 
        )
    )]
    pub async fn delete_by_id(
        id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                DELETE
                FROM sessions
                WHERE id = $1
            "#,
            id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions row database deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Delete all sessions from the database associated with a specific user.
    ///
    /// Executes a SQL `DELETE` statement to remove all session records that match the provided `user_id`.
    ///
    /// # Parameters
    /// * `user_id` - The UUID of the user whose sessions should be deleted.
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of rows deleted (equal to the number of sessions for the user, or 0 if none existed).
    /// * `Err(AuthenticationError)` - If the database operation fails.
    ///
    /// # Tracing
    /// - Adds the `user_id` to the tracing span for observability.
    #[tracing::instrument(
        name = "Delete all Users Sessions from the database: ",
        skip(database),
        fields(
            user_id = ?user_id
        )
    )]
    pub async fn delete_all_user(
        user_id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                DELETE
                FROM sessions
                WHERE user_id = $1
            "#,
            user_id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions database records deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Delete all sessions from the database.
    ///
    /// Executes a SQL `DELETE` statement to remove all session records from the `sessions` table.
    ///
    /// # Parameters
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of rows deleted (equal to the number of sessions deleted, or 0 if the table was already empty).
    /// * `Err(AuthenticationError)` - If the database operation fails.
    ///
    /// # Tracing
    /// - Adds a tracing span for observability.
    #[tracing::instrument(
        name = "Delete all Sessions from the database: ",
        skip(database)
    )]
    pub async fn delete_all(database: &Pool<Postgres>) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                DELETE
                FROM sessions

            "#,
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions rows deleted from the database: {rows_affected:#?}");

        Ok(rows_affected)
    }
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

    // Bring module functions into test scope
    use super::*;

    use fake::Fake;
    use sqlx::{Pool, Postgres};

    use crate::database;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    #[sqlx::test]
    async fn delete_self(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        let random_user = random_user.insert(&database).await?;

        // Generate sessions
        let session = database::Sessions::mock_data(&random_user).await?;

        // Insert session in the database for deleting
        let _database_record = session.insert(&database).await?;

        //-- Execute Function (Act)
        // Delete self from the database
        let rows_affected = session.delete(&database).await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 1);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn delete_by_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        let random_user = random_user.insert(&database).await?;

        // Generate session
        let session = database::Sessions::mock_data(&random_user).await?;

        // Insert session in the database for deleting
        let session = session.insert(&database).await?;

        //-- Execute Function (Act)
        // Delete database row
        let rows_affected =
            database::Sessions::delete_by_id(&session.id, &database).await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        // Each id is unique to the row, so I should equal 1
        assert_eq!(rows_affected, 1);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn delete_associated_to_user(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        let random_user = random_user.insert(&database).await?;

        // Add a random number of sessions for the given user
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate session
            let sessions = database::Sessions::mock_data(&random_user).await?;

            // Insert session into the database for deleting
            let _sessions = sessions.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Delete all database entries for the random user id
        let rows_affected =
            database::Sessions::delete_all_user(&random_user.id, &database)
                .await?;
        // println!("{rows_affected:#?}");

        //-- Checks (Assertions)
        // Rows affected should equal the random number of rows inserted
        assert_eq!(rows_affected, random_count as u64);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn delete_all(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate a random number of session
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate random user for testing
            let random_user = database::Users::mock_data()?;

            // Insert user in the database
            let random_user = random_user.insert(&database).await?;

            // Generate session
            let sessions = database::Sessions::mock_data(&random_user).await?;

            // Insert session in the database for deleting
            let _sessions = sessions.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Delete all rows in the database table
        let rows_affected = database::Sessions::delete_all(&database).await?;

        //-- Checks (Assertions)
        // Rows affected should equal the number of random entries
        assert_eq!(rows_affected, random_count as u64);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn delete_nonexistent_session_returns_zero(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // None

        //-- Execute Function (Act)
        // Try to delete a session that does not exist
        let random_id = Uuid::new_v4();
        let rows_affected = database::Sessions::delete_by_id(&random_id, &database).await?;

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 0);

        Ok(())
    }


    #[sqlx::test]
    async fn delete_nonexistent_user_sessions_returns_zero(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Try to delete all sessions for a user that does not exist
        let random_user = database::Users::mock_data()?;
        let random_user_id = random_user.id;

        //-- Execute Function (Act)
        let rows_affected = database::Sessions::delete_all_user(&random_user_id, &database).await?;

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 0);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_all_when_empty_returns_zero(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Ensure the sessions table is empty
        sqlx::query!("TRUNCATE TABLE sessions CASCADE").execute(&database).await?;

        //-- Execute Function (Act)
        let rows_affected = database::Sessions::delete_all(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 0);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_session_twice_returns_zero_on_second_try(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Setup: Insert a user and a session
        let random_user = database::Users::mock_data()?.insert(&database).await?;

        //-- Execute Function (Act)
        let session = database::Sessions::mock_data(&random_user).await?.insert(&database).await?;

        //-- Checks (Assertions)
        // First delete should succeed
        let rows_affected_first = session.delete(&database).await?;
        assert_eq!(rows_affected_first, 1);

        // Second delete should return 0
        let rows_affected_second = session.delete(&database).await?;
        assert_eq!(rows_affected_second, 0);

        Ok(())
    }
}
