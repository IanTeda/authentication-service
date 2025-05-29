//-- ./src/database/users/update.rs

//! Update User in the database, returning a Result with a UserModel instance
//! ---

// #![allow(unused)] // For development only

use crate::{domain, prelude::*};
use crate::database::Users;

impl Users {
    /// Update a `User` into the database, returning result with a UserModel instance.
    ///
    /// # Parameters
    ///
    /// * `user` - A User instance
    /// * `database` - An Sqlx database connection pool
    /// ---
    #[tracing::instrument(
        name = "Update a User in the database: ",
        skip(database),
        fields(
            user = ?self
        )
    )]
    pub async fn update(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Users, AuthenticationError> {
        let database_record = sqlx::query_as!(
			Users,
			r#"
				UPDATE users
				SET email = $2, name = $3, password_hash = $4, role = $5, is_active = $6, is_verified = $7
				WHERE id = $1
				RETURNING id, email, name, password_hash, role as "role:domain::UserRole", is_active, is_verified, created_on
			"#,
			self.id,
			self.email.as_ref(),
			self.name.as_ref(),
			self.password_hash.as_ref(),
			self.role.clone() as domain::UserRole,
			self.is_active,
			self.is_verified,
		)
            .fetch_one(database)
            .await?;

        tracing::debug!("User database records retrieved: {database_record:#?}");

        Ok(database_record)
    }
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {
    // use super::*;
    use sqlx::{Pool, Postgres};
    use crate::database;

    pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

    #[sqlx::test]
    async fn update_existing_user(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Create a user and insert it into the database     
        let original = database::Users::mock_data()?;
        original.insert(&database).await?;

        let mut updated = database::Users::mock_data()?;
        updated.id = original.id;
        updated.created_on = original.created_on;

        //-- Execute Function (Act)
        // Update the user in the database
        let db_user = updated.update(&database).await?;

        //-- Assert (Assert)
        // Check that the updated user matches the expected values
        assert_eq!(db_user, updated);

        Ok(())
    }

    #[sqlx::test]
    async fn update_nonexistent_user_returns_error(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = database::Users::mock_data()?;

        //-- Execute Function (Act)
        // Attempt to update a user that does not exist in the database
        let result = user.update(&database).await;

        //-- Assert (Assert)
        // Check that the update operation fails
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn update_to_duplicate_email_fails(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Create two users with different emails. The second user will attempt to 
        // update to the first user's email
        let user1 = database::Users::mock_data()?;
        let user2 = database::Users::mock_data()?;
        user1.insert(&database).await?;
        user2.insert(&database).await?;

        let mut user2_updated = user2.clone();
        user2_updated.email = user1.email.clone();

        //-- Execute Function (Act)
        // Attempt to update the second user to have the same email as the first user.
        // This should fail due to a unique constraint violation
        let result = user2_updated.update(&database).await;

        //-- Assert (Assert)
        // Assert that the update operation fails due to duplicate email
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn update_with_no_changes_succeeds(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Create a user, insert it into the database, and then update it without changes
        // This should succeed and return the same user instance
        let user = database::Users::mock_data()?;
        user.insert(&database).await?;

        //-- Execute Function (Act)
        // Update the user without changing any fields. This should return the same 
        // user instance as the one we inserted
        let db_user = user.update(&database).await?;

        //-- Assert (Assert)
        // Check that the user returned from the database matches the original user
        assert_eq!(db_user, user);
        Ok(())
    }

    #[sqlx::test]
    async fn update_preserves_id_and_created_on(database: Pool<Postgres>) -> Result<()> {

        let original = database::Users::mock_data()?;
        original.insert(&database).await?;

        let mut updated = database::Users::mock_data()?;
        updated.id = original.id;
        updated.created_on = original.created_on;

        //-- Execute Function (Act)
        // Update the user in the database. This should preserve the original ID 
        // and created_on timestamp.
        let db_user = updated.update(&database).await?;

        //-- Assert (Assert)
        // Check that the ID and created_on timestamp are the same as the original
        assert_eq!(db_user.id, original.id);
        assert_eq!(db_user.created_on, original.created_on);

        Ok(())
    }
}
