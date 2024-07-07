//-- ./src/database/refresh_token/delete.rs

//! Delete RefreshToken in the database, returning a Result with a boolean
//! ---

// #![allow(unused)] // For development only

use crate::prelude::*;

use uuid::Uuid;

/// Delete a RefreshToken from the database by querying the RefreshToken uuid, returning a Result
/// with a boolean or an sqlx error.
///
/// # Parameters
///
/// * `id` - The uuid of the refresh token to be deleted.
/// * `database` - An sqlx database pool that the thing will be searched in.
/// ---
#[tracing::instrument(
	name = "Delete a Refresh from the database using its id (uuid)."
	skip(id, database)
)]
pub async fn delete_refresh_token_by_id(
	id: &Uuid,
	database: &sqlx::Pool<sqlx::Postgres>,
) -> Result<bool, BackendError> {
	let rows_affected = sqlx::query!(
		r#"
			Delete 
			FROM refresh_tokens 
			WHERE id = $1
		"#,
		id
	)
	.execute(database)
	.await?
	.rows_affected();

	let confirm_deleted: bool = rows_affected != 0;

	tracing::debug!("RefreshToken record retrieved form database: {rows_affected:#?}");

	Ok(confirm_deleted)
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	// Bring module functions into test scope
	use super::*;

	use crate::database::{
		refresh_token::{insert_refresh_token, model::tests::generate_random_refresh_token}, 
		users::{insert_user, model::tests::generate_random_user}
	};

	use sqlx::{Pool, Postgres};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	// Test getting user from database using unique UUID
	#[sqlx::test]
	async fn delete_user_record_by_id(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate random user
		let random_user = generate_random_user()?;

		// Insert random user into the database
		insert_user(&random_user, &database).await?;

		// Generate refresh token
		let refresh_token = generate_random_refresh_token(random_user.id)?;
		// Insert refresh token in the database for deleting
		insert_refresh_token(&refresh_token, &database).await?;

		//-- Execute Function (Act)
		// Insert user into database
		let database_record = delete_refresh_token_by_id(&refresh_token.id, &database).await?;
		// println!("{record:#?}");

		//-- Checks (Assertions)
		assert!(database_record);

		// -- Return
		Ok(())
	}

	// Test deleting refresh token from the database using unique UUID
	#[sqlx::test]
	async fn delete_user_false(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate random user
		let random_user = generate_random_user()?;

		// Insert random user into the database
		insert_user(&random_user, &database).await?;

		// Generate refresh token
		let refresh_token = generate_random_refresh_token(random_user.id)?;
		// Insert refresh token in the database for deleting
		insert_refresh_token(&refresh_token, &database).await?;

		//-- Execute Function (Act)
		// Generate a new uuid
		let random_uuid = Uuid::now_v7();
		// Delete refresh token into database
		let database_record = delete_refresh_token_by_id(&random_uuid, &database).await?;
		// println!("{record:#?}");

		//-- Checks (Assertions)
		assert!(!database_record);

		// -- Return
		Ok(())
	}
}
