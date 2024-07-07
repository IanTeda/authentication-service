//-- ./src/database/refresh_token/create.rs

//! Create [insert] a Refresh token into the database, returning a Result with a RefreshTokenMoel instance
//! ---

// #![allow(unused)] // For development only

use crate:: prelude::*;

use super::model::RefreshTokenModel;

/// Insert a `RefreshToken` into the database, returning the database record
/// as a RefreshTokenModel.
///
/// # Parameters
///
/// * `refresh_token` - A Refresh Token instance
/// * `database` - An Sqlx database connection pool
/// ---
#[tracing::instrument(
	name = "Insert a new Refresh Token into the database."
	skip(refresh_token, database)
)]
pub async fn insert_refresh_token(
	refresh_token: &RefreshTokenModel,
	database: &sqlx::Pool<sqlx::Postgres>,
) -> Result<RefreshTokenModel, BackendError> {
	let created_refresh_token = sqlx::query_as!(
		RefreshTokenModel,
		r#"
            INSERT INTO refresh_tokens (id, user_id, refresh_token, is_active, created_on) 
            VALUES ($1, $2, $3, $4, $5) 
            RETURNING *
        "#,
		refresh_token.id,
		refresh_token.user_id,
		refresh_token.refresh_token,
		refresh_token.is_active,
		refresh_token.created_on
	)
	.fetch_one(database)
	.await?;

	tracing::debug!("Record inserted into database: {created_refresh_token:#?}");

	Ok(created_refresh_token)
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	// Bring module functions into test scope
	use super::*;
	use crate::database::{
		refresh_token::model::tests::generate_random_refresh_token, 
		users::{insert_user, model::tests::generate_random_user}
	};

	use rand::random;
	use sqlx::{Pool, Postgres};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	// Test inserting into database
	#[sqlx::test]
	async fn create_database_record(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate random user
		let random_user = generate_random_user()?;

		// Insert random user into the database
		insert_user(&random_user, &database).await?;

		// Generate refresh token
		let refresh_token = generate_random_refresh_token(random_user.id)?;

		//-- Execute Function (Act)
		// Insert refresh token into database
		let database_record = insert_refresh_token(&refresh_token, &database).await?;
		// println!("{database_record:#?}");

		//-- Checks (Assertions)
		assert_eq!(database_record.id,refresh_token.id);
		assert_eq!(database_record.user_id, random_user.id);
		assert_eq!(database_record.refresh_token, refresh_token.refresh_token);
		assert_eq!(database_record.is_active, refresh_token.is_active);
		assert_eq!(
			database_record.created_on.timestamp(),
			refresh_token.created_on.timestamp()
		);

		// -- Return
		Ok(())
	}
}
