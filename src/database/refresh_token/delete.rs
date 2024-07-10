//-- ./src/database/refresh_token/delete.rs

//! Delete RefreshToken in the database, returning a Result with a u64 of the number
//! of rows affected.
//! 
//! This module adds two impl methods to RefreshTokenModel:
//! 
//! 1. `delete`: Delete Refresh Token instance in the database, returning the number of rows affected.
//! 2. `delete_all_user_id`: Delete all Refresh Token instances for a given user_id, returning the number of rows affected
//! ---

// #![allow(unused)] // For development only

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::prelude::*;

impl super::RefreshTokenModel {
	/// Delete a Refresh Token from the database by querying the Refresh Token uuid, returning a Result
	/// with the number of rows deleted or an sqlx error.
	///
	/// # Parameters
	///
	/// * `self` - The Refresh Token instance to be deleted.
	/// * `database` - An sqlx database pool that the thing will be searched in.
	/// ---
	#[tracing::instrument(
		name = "Delete a Refresh Token from the database using its id (uuid)."
		skip(self, database)
		fields(
        	db_id = %self.id,
			user_id = %self.user_id,
			refresh_token = %self.refresh_token.as_ref(),
			is_active = %self.is_active,
			created_on = %self.created_on,
    	)
	)]
	pub async fn delete(
		&self,
		database: &Pool<Postgres>,
	) -> Result<u64, BackendError> {
		let rows_affected = sqlx::query!(
			r#"
				Delete 
				FROM refresh_tokens 
				WHERE id = $1
			"#,
			self.id
		)
		.execute(database)
		.await?
		.rows_affected();

		Ok(rows_affected)
	}

	/// Delete all Refresh Tokens from the database by querying the Refresh Token user_id, returning a Result
	/// with the u64 number of rows deleted or an sqlx error.
	///
	/// # Parameters
	///
	/// * `user_id` - The user_id for the Refresh Tokens to be deleted
	/// * `database` - An sqlx database pool that the thing will be searched in.
	/// ---
	#[tracing::instrument(
		name = "Delete a Refresh Token from the database using a user_id (uuid)."
		skip(user_id, database)
		fields(
			user_id = %user_id,
    	)
	)]
	pub async fn delete_all_user_id(
		user_id: &Uuid,
		database: &Pool<Postgres>,
	) -> Result<u64, BackendError> {
		let rows_affected = sqlx::query!(
			r#"
				Delete 
				FROM refresh_tokens 
				WHERE user_id = $1
			"#,
			user_id
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

	// Bring module functions into test scope
	// use super::*;

	use fake::Fake;
	use sqlx::{Pool, Postgres};

	use crate::database::{RefreshTokenModel, UserModel};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	// Test getting user from database using unique UUID
	#[sqlx::test]
	async fn delete_refresh_token_record(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_user = UserModel::generate_random().await?;

		// Insert user in the database
		random_user.insert(&database).await?;

		// Generate refresh token
		let refresh_token =
			RefreshTokenModel::create_random(&random_user.id).await?;

		// Insert refresh token in the database for deleting
		refresh_token.insert(&database).await?;

		//-- Execute Function (Act)
		// Insert user into database
		let rows_affected = refresh_token.delete(&database).await?;
		// println!("{record:#?}");

		//-- Checks (Assertions)
		assert!(rows_affected == 1);

		// -- Return
		Ok(())
	}

	// Test getting user from database using unique UUID
	#[sqlx::test]
	async fn delete_all_refresh_tokens_with_user_id(
		database: Pool<Postgres>,
	) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_user = UserModel::generate_random().await?;

		// Insert user in the database
		random_user.insert(&database).await?;

		let random_count: i64 = (10..30).fake::<i64>();
		for _count in 0..random_count {
			// Generate refresh token
			let refresh_token =
				RefreshTokenModel::create_random(&random_user.id).await?;

			// Insert refresh token in the database for deleting
			refresh_token.insert(&database).await?;
		}

		//-- Execute Function (Act)
		// Insert user into database
		let rows_affected = RefreshTokenModel::delete_all_user_id(&random_user.id, &database).await?;
		// println!("{rows_affected:#?}");

		//-- Checks (Assertions)
		assert!(rows_affected == random_count as u64);

		// -- Return
		Ok(())
	}
}
