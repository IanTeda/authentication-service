//-- ./src/database/refresh_token/update.rs

//! Update Refresh Token in the database
//!
//! This module has three impl functions for RefreshTokenModel:
//!
//!  1. `update`: Update the RefreshTokenModel instance in the database
//!  2. `revoke`: Revoke the RefreshTokenModel instance in the database
//!  3. `revoke_user_id`: Revoke all the RefreshTokenModel rows in the database with the user_id
//!
//! ---

// #![allow(unused)] // For development only

use crate::prelude::BackendError;

use super::model::RefreshTokenModel;

use sqlx::{Pool, Postgres};
use uuid::Uuid;

impl super::RefreshTokenModel {
	/// Update a `Refresh Token` in the database, returning a result with a RefreshTokenModel instance
	/// or Sqlx error.
	///
	/// # Parameters
	///
	/// * `self` - A RefreshTokenModel instance.
	/// * `database` - An Sqlx database connection pool.
	/// ---
	#[tracing::instrument(
		name = "Update a Refresh Token in the database."
		skip(self, database)
		fields(
        	db_id = %self.id,
			user_id = %self.user_id,
			refresh_token = %self.refresh_token.as_ref(),
			is_active = %self.is_active,
			created_on = %self.created_on,
    	)
	)]
	pub async fn update(
		&self,
		database: &Pool<Postgres>,
	) -> Result<RefreshTokenModel, BackendError> {
		let database_record = sqlx::query_as!(
			RefreshTokenModel,
			r#"
				UPDATE refresh_tokens 
				SET user_id = $2, refresh_token = $3, is_active = $4
				WHERE id = $1 
				RETURNING *
			"#,
			self.id,
			self.user_id,
			self.refresh_token.as_ref(),
			self.is_active,
		)
		.fetch_one(database)
		.await?;

		Ok(database_record)
	}

	/// Revoke (make non-active) a Refresh Token in the database, returning a result
	/// with a RefreshTokenModel instance or and SQLx error
	///
	/// # Parameters
	///
	/// * `self` - A RefreshTokenModel instance.
	/// * `database` - An Sqlx database connection pool.
	/// ---
	#[tracing::instrument(
		name = "Revoke a Refresh Token in the database."
		skip(self, database)
		fields(
        	db_id = %self.id,
			user_id = %self.user_id,
			refresh_token = %self.refresh_token.as_ref(),
			is_active = %self.is_active,
			created_on = %self.created_on,
    	)
	)]
	pub async fn revoke(
		&self,
		database: &Pool<Postgres>,
	) -> Result<Self, BackendError> {
		let database_record = sqlx::query_as!(
			RefreshTokenModel,
			r#"
				UPDATE refresh_tokens 
				SET is_active = false
				WHERE id = $1 
				RETURNING *
			"#,
			self.id
		)
		.fetch_one(database)
		.await?;

		Ok(database_record)
	}

	/// Revoke (make non-active) all Refresh Token in the database for a give user_id,
	/// returning a result with the number RefreshTokenModels revoked or an SQLx error
	///
	/// # Parameters
	///
	/// * `user_id` - The user_id for the Refresh Tokens to be revoked.
	/// * `database` - An Sqlx database connection pool.
	/// ---
	#[tracing::instrument(
		name = "Revoke all Refresh Tokens for a given user_id in the database."
		skip(database)
	)]
	pub async fn revoke_user_id(
		user_id: &Uuid,
		database: &Pool<Postgres>,
	) -> Result<u64, sqlx::Error> {
		let rows_affected = sqlx::query_as!(
			RefreshTokenModel,
			r#"
				UPDATE refresh_tokens 
				SET is_active = false
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

	use fake::Fake;
	use sqlx::{Pool, Postgres};

	use crate::database::{RefreshTokenModel, UserModel};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	#[sqlx::test]
	async fn update_refresh_token(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_user = UserModel::generate_random().await?;

		// Insert user in the database
		random_user.insert(&database).await?;

		// Generate refresh token
		let mut refresh_token =
			RefreshTokenModel::create_random(&random_user.id).await?;

		let original_refresh_token_id = refresh_token.id;
		let original_refresh_token_created_on = refresh_token.created_on;

		// Insert refresh token in the database for deleting
		refresh_token.insert(&database).await?;

		// Generate a new user and refresh token for updating
		let random_user_update = UserModel::generate_random().await?;

		// Insert user in the database
		random_user_update.insert(&database).await?;

		// Generate refresh token
		let refresh_token_update =
			RefreshTokenModel::create_random(&random_user_update.id).await?;

		// refresh_token.id = refresh_token_update.id;
		refresh_token.user_id = random_user_update.id;
		refresh_token.refresh_token = refresh_token_update.clone().refresh_token;
		refresh_token.is_active = refresh_token_update.clone().is_active;
		refresh_token.created_on = refresh_token_update.clone().created_on;

		//-- Execute Function (Act)
		// Generate an updated Refresh Token
		let database_record = refresh_token.clone().update(&database).await?;

		//-- Checks (Assertions)
		assert_eq!(database_record.id, original_refresh_token_id);
		assert_eq!(database_record.user_id, random_user_update.id);
		assert_eq!(
			database_record.refresh_token,
			refresh_token_update.refresh_token
		);
		assert_eq!(database_record.is_active, refresh_token_update.is_active);
		assert_eq!(
			database_record.created_on.timestamp(),
			original_refresh_token_created_on.timestamp()
		);

		// -- Return
		Ok(())
	}

	#[sqlx::test]
	async fn revoke_refresh_token(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_user = UserModel::generate_random().await?;

		// Insert user in the database
		random_user.insert(&database).await?;

		// Generate refresh token
		let mut refresh_token =
			RefreshTokenModel::create_random(&random_user.id).await?;

		// Set Refresh Token active to true
		refresh_token.is_active = true;

		// Insert Refresh Token into database
		refresh_token.insert(&database).await?;

		// Insert refresh token in the database for deleting
		refresh_token.revoke(&database).await?;

		//-- Execute Function (Act)
		// Generate an updated Refresh Token
		let database_record =
			RefreshTokenModel::from_id(&refresh_token.id, &database).await?;

		//-- Checks (Assertions)
		assert_eq!(database_record.is_active, false);

		// -- Return
		Ok(())
	}

	#[sqlx::test]
	async fn revoke_all_user_refresh_tokens(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_user = UserModel::generate_random().await?;

		// Insert user in the database
		random_user.insert(&database).await?;

		let mut test_vec: Vec<RefreshTokenModel> = Vec::new();
		let random_count: i64 = (10..30).fake::<i64>();
		for _count in 0..random_count {
			// Generate refresh token
			let mut refresh_token =
				RefreshTokenModel::create_random(&random_user.id).await?;

			// Set Refresh Token active to true
			refresh_token.is_active = true;

			// Insert refresh token in the database for deleting
			refresh_token.insert(&database).await?;

			// Add Refresh Token to a Vec
			test_vec.push(refresh_token);
		}

		//-- Execute Function (Act)
		// Generate an updated Refresh Token
		let rows_affected =
			RefreshTokenModel::revoke_user_id(&random_user.id, &database).await?;

		//-- Checks (Assertions)
		assert!(rows_affected == random_count as u64);

		// Pick a random Refresh Token to asset is_active as false
		let random_index_number = (1..random_count).fake::<i64>() as usize;

		// Get the database record
		let database_record =
			RefreshTokenModel::from_id(&test_vec[random_index_number].id, &database)
				.await?;

		// Check the is_active status is false
		assert_eq!(database_record.is_active, false);

		Ok(())
	}
}
