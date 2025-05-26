//-- ./src/database/users/delete.rs

//! Delete User in the database, returning a Result with a boolean
//! ---

// #![allow(unused)] // For development only

use crate::database::Users;
use crate::prelude::*;

impl Users {
    /// Delete a User from the database, returning a Result with the number of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `self` - The User instance to be deleted from the database
    /// * `database` - The sqlx database pool that the User will be deleted from.
    /// ---
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
    use sqlx::{Pool, Postgres};
    use crate::database;

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
        let fetch_result = database::Users::from_user_id(&random_test_user.id, &database).await;
        assert!(fetch_result.is_err());

        // -- Return
        Ok(())
    }

    // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn delete_nonexistent_user(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let mut random_test_user = database::Users::mock_data()?;

        // Don not insert user in the database

        //-- Execute Function (Act)
        // Insert user into database
        let rows_affected = random_test_user.delete(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 0);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn delete_user_twice(database: Pool<Postgres>) -> Result<()> {
        // Arrange
        let random_test_user = database::Users::mock_data()?;
        random_test_user.insert(&database).await?;

        // Act
        let first_delete = random_test_user.delete(&database).await?;
        let second_delete = random_test_user.delete(&database).await?;

        // Assert
        assert_eq!(first_delete, 1);
        assert_eq!(second_delete, 0);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_multiple_users(database: Pool<Postgres>) -> Result<()> {
        // Arrange
        let user1 = database::Users::mock_data()?;
        let user2 = database::Users::mock_data()?;
        user1.insert(&database).await?;
        user2.insert(&database).await?;

        // Act
        let rows_affected1 = user1.delete(&database).await?;
        let rows_affected2 = user2.delete(&database).await?;

        // Assert
        assert_eq!(rows_affected1, 1);
        assert_eq!(rows_affected2, 1);
        Ok(())
    }
}
