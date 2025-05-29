//-- ./src/database/users/delete.rs

//! User deletion logic for the authentication service.
//!
//! This module provides functions to delete user records from the database,
//! including deletion by user instance and comprehensive unit tests for deletion scenarios.
//!
//! # Contents
//! - Delete a single user by instance
//! - Unit tests for user deletion logic and edge cases

// #![allow(unused)] // For development only

use crate::database::Users;
use crate::prelude::*;

impl Users {
    /// Delete this user from the database.
    ///
    /// Executes a SQL `DELETE` statement to remove the user record identified by its `id`.
    ///
    /// # Parameters
    /// * `self` - The `Users` instance to be deleted.
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of rows deleted (should be 1 if the user existed, 0 otherwise).
    /// * `Err(AuthenticationError)` - If the database operation fails.
    ///
    /// # Tracing
    /// - Adds the user `id` to the tracing span for observability.
    #[tracing::instrument(
        name = "Delete a User from the database with id: ",
        skip(self, database),
        fields(
            user_id = ?self.id,
        )
    )]
    pub async fn delete(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                DELETE
                FROM users
                WHERE id = $1
            "#,
            self.id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("User database records affected: {rows_affected:#?}");

        Ok(rows_affected)
    }
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {
    use crate::database;
    use sqlx::{Pool, Postgres};

    // Bring module functions into test scope
    // use super::*;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn delete_existing_user(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_test_user = database::Users::mock_data()?;

        // Insert user in the database
        random_test_user.insert(&database).await?;

        //-- Execute Function (Act)
        // Delete user in the database
        let rows_affected = random_test_user.delete(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 1);

        // Try to fetch the user, should fail
        let fetch_result =
            database::Users::from_user_id(&random_test_user.id, &database).await;
        assert!(fetch_result.is_err());

        // -- Return
        Ok(())
    }

    // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn delete_nonexistent_user(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_test_user = database::Users::mock_data()?;

        // Don not insert user in the database

        //-- Execute Function (Act)
        // Delete user into database
        let rows_affected = random_test_user.delete(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 0);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn delete_user_twice(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_test_user = database::Users::mock_data()?;
        random_test_user.insert(&database).await?;

        //-- Execute Function (Act)
        let first_delete = random_test_user.delete(&database).await?;
        let second_delete = random_test_user.delete(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(first_delete, 1);
        assert_eq!(second_delete, 0);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_multiple_users(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user1 = database::Users::mock_data()?;
        let user2 = database::Users::mock_data()?;
        user1.insert(&database).await?;
        user2.insert(&database).await?;

        //-- Execute Function (Act)
        let rows_affected1 = user1.delete(&database).await?;
        let rows_affected2 = user2.delete(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(rows_affected1, 1);
        assert_eq!(rows_affected2, 1);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_user_with_sessions_cascades_or_fails(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Arrange: Insert a user and a session for that user
        let user = database::Users::mock_data()?;
        let user = user.insert(&database).await?;
        let session = database::Sessions::mock_data(&user).await?;
        session.insert(&database).await?;

        //-- Execute Function (Act)
        // Act: Delete the user
        let rows_affected = user.delete(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 1);

        // Try to fetch the user, should fail
        let fetch_result = database::Users::from_user_id(&user.id, &database).await;
        assert!(fetch_result.is_err());

        // Try to fetch the session, should fail if ON DELETE CASCADE is set, or still exist otherwise
        let session_result =
            database::Sessions::from_id(&session.id, &database).await;
        // If you expect cascade, assert error; otherwise, comment this out or adjust as needed
        assert!(session_result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn delete_all_users(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Arrange: Insert multiple users
        let users = database::Users::insert_n_users(5, &database).await?;
        // Act: Delete all users one by one
        let mut total_deleted = 0;
        for user in &users {
            total_deleted += user.delete(&database).await?;
        }

        //-- Execute Function (Act)
        // Assert: All users deleted
        assert_eq!(total_deleted, users.len() as u64);

        // Try to fetch any user, should fail
        for user in &users {
            let fetch_result =
                database::Users::from_user_id(&user.id, &database).await;
            //-- Checks (Assertions)
            assert!(fetch_result.is_err());
        }
        Ok(())
    }

    #[sqlx::test]
    async fn delete_user_invalid_id_returns_zero(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        use uuid::Uuid;
        // Arrange: Create a user struct with a random UUID that does not exist in DB
        let mut user = database::Users::mock_data()?;
        user.id = Uuid::new_v4();

        //-- Execute Function (Act)
        // Act: Attempt to delete
        let rows_affected = user.delete(&database).await?;

        //-- Checks (Assertions)
        // Assert: Should return 0
        assert_eq!(rows_affected, 0);
        Ok(())
    }
}
