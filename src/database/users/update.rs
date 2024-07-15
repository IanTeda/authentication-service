//-- ./src/database/users/update.rs

//! Update User in the database, returning a Result with a UserModel instance
//! ---

// #![allow(unused)] // For development only

use crate::database::Users;
use crate::{domain, prelude::*};

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
        skip(self, database)
    )]
    pub async fn update(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Users, BackendError> {
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
    use sqlx::{Pool, Postgres};

    use crate::database;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    // Test inserting into database
    #[cfg(feature = "mocks")]
    #[sqlx::test]
    async fn create_database_record(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let original_test_user = database::Users::mock_data()?;

        // Insert user in the database
        original_test_user.insert(&database).await?;

        // Generate new data for updating the database
        let mut updated_test_user = database::Users::mock_data()?;
        updated_test_user.id = original_test_user.id;
        updated_test_user.created_on = original_test_user.created_on;

        //-- Execute Function (Act)
        // Insert user into database
        let database_record = updated_test_user.update(&database).await?;
        // println!("{updated_user:#?}");

        //-- Checks (Assertions)
        assert_eq!(database_record, updated_test_user);

        // -- Return
        Ok(())
    }
}
