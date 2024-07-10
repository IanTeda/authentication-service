//-- ./src/database/users/update.rs

//! Update User in the database, returning a Result with a UserModel instance
//! ---

// #![allow(unused)] // For development only

use crate::prelude::*;

use super::UserModel;

impl super::UserModel {
	/// Update a `User` into the database, returning result with a UserModel instance.
	///
	/// # Parameters
	///
	/// * `user` - A User instance
	/// * `database` - An Sqlx database connection pool
	/// ---
	#[tracing::instrument(
		name = "Update a User in the database."
		skip(self, database)
	)]
	pub async fn update(
		&self,
		database: &sqlx::Pool<sqlx::Postgres>,
	) -> Result<UserModel, BackendError> {
		let updated_user = sqlx::query_as!(
			UserModel,
			r#"
				UPDATE users 
				SET email = $2, user_name = $3, password_hash = $4, is_active = $5 
				WHERE id = $1 
				RETURNING *
			"#,
			self.id,
			self.email.as_ref(),
			self.user_name.as_ref(),
			self.password_hash.as_ref(),
			self.is_active,
		)
		.fetch_one(database)
		.await?;

		Ok(updated_user)
	}
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	// Bring module functions into test scope
	use super::*;

	use sqlx::{Pool, Postgres};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	// Test inserting into database
	#[sqlx::test]
	async fn create_database_record(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let original_test_user = UserModel::generate_random().await?;

		// Insert user in the database
		original_test_user.insert(&database).await?;

		// Generate new data for updating the database
		let mut updated_test_user = UserModel::generate_random().await?;
		updated_test_user.id = original_test_user.id;
		updated_test_user.created_on = original_test_user.created_on;

		//-- Execute Function (Act)
		// Insert user into database
		let database_record = updated_test_user.update(&database).await?;
		// println!("{updated_user:#?}");

		//-- Checks (Assertions)
		assert_eq!(database_record.id, original_test_user.id);
		assert_eq!(database_record.email, updated_test_user.email);
		assert_eq!(database_record.user_name, updated_test_user.user_name);
		assert_eq!(
			database_record.password_hash,
			updated_test_user.password_hash
		);
		assert_eq!(database_record.is_active, updated_test_user.is_active);
		// Use timestamp because Postgres time precision is less than Rust
		assert_eq!(
			database_record.created_on.timestamp(),
			original_test_user.created_on.timestamp()
		);

		// -- Return
		Ok(())
	}
}
