//-- ./src/database/users/create.rs

//! Create [insert] User into the database, returning a Result with a UserModel instance
//! ---

// #![allow(unused)] // For development only

use crate::{
	database::users::UserModel,
	prelude::*
};

pub async fn create_user(
	user: &UserModel,
	database: &sqlx::Pool<sqlx::Postgres>,
) -> Result<UserModel, BackendError> {

	let created_user = sqlx::query_as!(
		UserModel,
		r#"
            INSERT INTO users (id, email, user_name, password_hash, is_active, created_on) 
            VALUES ($1, $2, $3, $4, $5, $6) 
            RETURNING *
        "#,
		user.id,
		user.email.as_ref(),
		user.user_name.as_ref(),
		user.password_hash,
		user.is_active,
		user.created_on
	)
	.fetch_one(database)
	.await?;
	
	tracing::debug!("Record inserted into database: {created_user:#?}");
    
	Ok(created_user)
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	// Bring module functions into test scope
	use super::*;

	use crate::database::users::model::tests::create_random_user;

	use sqlx::{Pool, Postgres};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	// Test inserting into database
	#[sqlx::test]
	async fn create_database_record(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_test_user = create_random_user()?;
		// println!("{test_thing:#?}");

		//-- Execute Function (Act)
		// Insert user into database
		let created_user = create_user(&random_test_user, &database).await?;
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