//-- ./src/database/refresh_token/update.rs

//! Update Refresh Token in the database, returning a Result with a RefreshTokenModel instance
//! ---

// #![allow(unused)] // For development only

use super::model::RefreshTokenModel;
use crate::prelude::*;

use uuid::Uuid;

/// Update a `Refresh Token` into the database, returning result with a RefreshTokenModel instance.
///
/// # Parameters
///
/// * `user` - A RefreshTokenModel instance
/// * `database` - An Sqlx database connection pool
/// ---
#[tracing::instrument(
	name = "Update a refresh Token in the database."
	skip(refresh_token, database)
)]
pub async fn update_refresh_token_by_id(
	refresh_token: &RefreshTokenModel,
	database: &sqlx::Pool<sqlx::Postgres>,
) -> Result<RefreshTokenModel, BackendError> {
	let updated_refresh_token = sqlx::query_as!(
		RefreshTokenModel,
		r#"
            UPDATE refresh_tokens 
            SET user_id = $2, refresh_token = $3, is_active = $4
            WHERE id = $1 
            RETURNING *
        "#,
		refresh_token.id,
		refresh_token.user_id,
		refresh_token.refresh_token,
		refresh_token.is_active,
	)
	.fetch_one(database)
	.await?;

	tracing::debug!("Refresh Token updated in the database: {updated_refresh_token:#?}");

	Ok(updated_refresh_token)
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	// Bring module functions into test scope
	use super::*;

	use crate::database::{
		refresh_token::{insert_refresh_token, model::tests::generate_random_refresh_token},
		users::{insert_user, model::tests::generate_random_user},
	};

	use fake::Fake;
	use sqlx::{Pool, Postgres};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	// Test inserting into database
	#[sqlx::test]
	async fn update_database_record(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate random user
		let random_user = generate_random_user()?;

		// Insert random user into the database
		insert_user(&random_user, &database).await?;

		// Generate refresh token
		let mut original_refresh_token = generate_random_refresh_token(random_user.id)?;

		// Insert refresh token in the database for deleting
		insert_refresh_token(&original_refresh_token, &database).await?;


		//-- Execute Function (Act)
		// Generate an updated Refresh Token
		let mut updated_refresh_token = generate_random_refresh_token(random_user.id)?;
		updated_refresh_token.id = original_refresh_token.id;

		// Get the Refresh Tokens
		let database_record = update_refresh_token_by_id(&updated_refresh_token, &database).await?;
		// println!("{record:#?}");


		//-- Checks (Assertions)
		assert_eq!(database_record.id, original_refresh_token.id);
		assert_eq!(database_record.user_id, random_user.id);
		assert_eq!(database_record.refresh_token, updated_refresh_token.refresh_token);
		assert_eq!(database_record.is_active, updated_refresh_token.is_active);
		assert_eq!(
			database_record.created_on.timestamp(),
			original_refresh_token.created_on.timestamp()
		);

		// -- Return
		Ok(())
	}
}
