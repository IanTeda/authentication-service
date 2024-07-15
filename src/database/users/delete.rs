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
            user_id = % self.id,
        )
    )]
    pub async fn delete(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
					Delete
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
    #[cfg(feature = "mocks")]
    #[sqlx::test]
    async fn delete_user_record(database: Pool<Postgres>) -> Result<()> {
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

        // -- Return
        Ok(())
    }

    // Test getting user from database using unique UUID
    #[cfg(feature = "mocks")]
    #[sqlx::test]
    async fn delete_user_false(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let mut random_test_user = database::Users::mock_data()?;

        // Insert user in the database
        random_test_user.insert(&database).await?;

        // Generate a new random user id and push to instance for testing
        random_test_user.id = database::Users::mock_data()?.id;

        //-- Execute Function (Act)
        // Insert user into database
        let rows_affected = random_test_user.delete(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 0);

        // -- Return
        Ok(())
    }
}
