//-- ./src/database/users/update.rs

//! Update User in the database, returning a Result with a UserModel instance
//! ---

// #![allow(unused)] // For development only

use crate::{database::users::UserModel, prelude::*};

/// Update a `User` into the database, returning result with a UserModel instance.
///
/// # Parameters
///
/// * `user` - A User instance
/// * `database` - An Sqlx database connection pool
/// ---
#[tracing::instrument(
	name = "Update a new User into the database."
	skip(user, database)
)]
pub async fn update_user_by_id(
	user: &UserModel,
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
		user.id,
		user.email.as_ref(),
		user.user_name.as_ref(),
		user.password_hash,
		user.is_active,
	)
	.fetch_one(database)
	.await?;

	tracing::debug!("Record inserted into database: {updated_user:#?}");

	Ok(updated_user)
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	// Bring module functions into test scope
	use super::*;

	use crate::database::users::{insert_user, model::tests::generate_random_user};

	use sqlx::{Pool, Postgres};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	// Test inserting into database
	#[sqlx::test]
	async fn create_database_record(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate random user for testing
		let original_random_test_user = generate_random_user()?;
		insert_user(&original_random_test_user, &database).await?;

		// Generate new data for updating the database
		let random_test_user = generate_random_user()?;
		let mut updated_random_test_user = original_random_test_user.clone();
		updated_random_test_user.email = random_test_user.email;
		updated_random_test_user.user_name = random_test_user.user_name;
		updated_random_test_user.password_hash = random_test_user.password_hash;
		updated_random_test_user.is_active = random_test_user.is_active;

		//-- Execute Function (Act)
		// Insert user into database
		let updated_user = update_user_by_id(&updated_random_test_user, &database).await?;
		// println!("{updated_user:#?}");

		//-- Checks (Assertions)
		assert_eq!(updated_user.id, original_random_test_user.id);
		assert_eq!(updated_user.email, updated_random_test_user.email);
		assert_eq!(updated_user.user_name, updated_random_test_user.user_name);
		assert_eq!(
			updated_user.password_hash,
			updated_random_test_user.password_hash
		);
		assert_eq!(updated_user.is_active, updated_random_test_user.is_active);
		// Use timestamp because Postgres time precision is less than Rust
		assert_eq!(
			updated_user.created_on.timestamp(),
			original_random_test_user.created_on.timestamp()
		);

		// -- Return
		Ok(())
	}
}
