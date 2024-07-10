//-- ./src/database/users/delete.rs

//! Delete User in the database, returning a Result with a boolean
//! ---

// #![allow(unused)] // For development only

use crate::prelude::*;

impl super::UserModel {
	/// Delete a User Model instance from the database, returning a Result
	/// with the number of rows deleted or an sqlx error.
	///
	/// # Parameters
	///
	/// * `self` - The User Model instance to be deleted from the database
	/// * `database` - The sqlx database pool that the User will be deleted from.
	/// ---
	#[tracing::instrument(
		name = "Delete a User from the database."
		skip(self, database)
		fields(
			id = %self.id,
			email = %self.email,
			user_name = %self.user_name.as_ref(),
			password_hash = %self.password_hash.as_ref(),
			is_active = %self.is_active,
			created_on = %self.created_on,
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

		Ok(rows_affected)
	}
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	use crate::database::UserModel;

	// Bring module functions into test scope
	// use super::*;

	use sqlx::{Pool, Postgres};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	// Test getting user from database using unique UUID
	#[sqlx::test]
	async fn delete_user_record(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_test_user = UserModel::generate_random().await?;

		// Insert user in the database
		random_test_user.insert(&database).await?;

		//-- Execute Function (Act)
		// Delete user in the database
		let rows_affected = random_test_user.delete(&database).await?;

		//-- Checks (Assertions)
		assert!(rows_affected == 1);

		// -- Return
		Ok(())
	}

	// Test getting user from database using unique UUID
	#[sqlx::test]
	async fn delete_user_false(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let mut random_test_user = UserModel::generate_random().await?;

		// Insert user in the database
		random_test_user.insert(&database).await?;

		// Generate a new random user id and push to instance for testing
		random_test_user.id = UserModel::generate_random().await?.id;

		//-- Execute Function (Act)
		// Insert user into database
		let rows_affected = random_test_user.delete(&database).await?;

		//-- Checks (Assertions)
		assert!(rows_affected == 0);

		// -- Return
		Ok(())
	}
}
