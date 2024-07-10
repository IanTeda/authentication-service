//-- ./src/database/users/insert

//! Insert User instance into the database, returning a Result with a User Model instance
//! ---

// #![allow(unused)] // For development only

use crate::{database::users::UserModel, prelude::*};

impl super::UserModel {
	/// Insert a `User` into the database, returning a result with the User Model instance created.
	///
	/// # Parameters
	///
	/// * `self` - The User Model instance to be inserted in the database.
	/// * `database` - An Sqlx database connection pool
	/// ---
	#[tracing::instrument(
		name = "Insert a new User into the database."
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
	pub async fn insert(
		&self,
		database: &sqlx::Pool<sqlx::Postgres>,
	) -> Result<UserModel, BackendError> {
		let created_user = sqlx::query_as!(
			UserModel,
			r#"
				INSERT INTO users (id, email, user_name, password_hash, is_active, created_on)
				VALUES ($1, $2, $3, $4, $5, $6)
				RETURNING *
			"#,
			self.id,
			self.email.as_ref(),
			self.user_name.as_ref(),
			self.password_hash.as_ref(),
			self.is_active,
			self.created_on
		)
		.fetch_one(database)
		.await?;

		Ok(created_user)
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
		let random_test_user = UserModel::generate_random().await?;

		//-- Execute Function (Act)
		// Insert user into database
		let created_user = random_test_user.insert(&database).await?;
		// println!("{record:#?}");

		//-- Checks (Assertions)
		assert_eq!(created_user.id, random_test_user.id);
		assert_eq!(created_user.email, random_test_user.email);
		assert_eq!(created_user.user_name, random_test_user.user_name);
		assert_eq!(created_user.password_hash, random_test_user.password_hash);
		assert_eq!(created_user.is_active, random_test_user.is_active);
		// Use timestamp because Postgres time precision is less than Rust
		assert_eq!(
			created_user.created_on.timestamp(),
			random_test_user.created_on.timestamp()
		);

		// -- Return
		Ok(())
	}
}
