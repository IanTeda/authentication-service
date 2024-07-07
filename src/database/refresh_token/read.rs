//-- ./src/database/refresh_tokens/read.rs

//! Read RefreshToken[s] in the database, returning a Result with a RefreshTokenModel instance
//! or a Vec[RefreshTokenModel]
//! ---

// // #![allow(unused)] // For development only

use super::model::RefreshTokenModel;
use crate::prelude::*;

use uuid::Uuid;

/// Get a Refresh Token from the database by querying the uuid, returning a Refresh Token instance or sqlx error.
///
/// # Parameters
///
/// * `id` - The uuid of the Refresh Token to be returned.
/// * `database` - An sqlx database pool that the thing will be searched in.
/// ---
#[tracing::instrument(
	name = "Get a Refresh Token from the database using its id (uuid)."
	skip(id, database)
)]
pub async fn select_refresh_token_by_id(
	id: &Uuid,
	database: &sqlx::Pool<sqlx::Postgres>,
) -> Result<RefreshTokenModel, BackendError> {
	let refresh_token_record = sqlx::query_as!(
		RefreshTokenModel,
		r#"
			SELECT * 
			FROM refresh_tokens 
			WHERE id = $1
		"#,
		id
	)
	.fetch_one(database)
	.await?;

	tracing::debug!("Refresh Token record retrieved form database: {refresh_token_record:#?}");

	Ok(refresh_token_record)
}

/// Get Refresh Token from the database by querying the Users uuid,
/// returning a Refresh Token instance or sqlx error.
///
/// # Parameters
///
/// * `id` - The uuid of user to be returned
/// * `database` - An sqlx database pool that the thing will be searched in.
#[tracing::instrument(
	name = "Get a Refresh Token from the database using users id (uuid)."
	skip(user_id, database)
)]
pub async fn select_refresh_token_by_user_id(
	user_id: &Uuid,
	database: &sqlx::Pool<sqlx::Postgres>,
) -> Result<Vec<RefreshTokenModel>, BackendError> {
	let refresh_token_records = sqlx::query_as!(
		RefreshTokenModel,
		r#"
			SELECT * 
			FROM refresh_tokens 
			WHERE user_id = $1
		"#,
		user_id
	)
	.fetch_all(database)
	.await?;

	tracing::debug!("Refresh token records retrieved form database: {refresh_token_records:#?}");

	Ok(refresh_token_records)
}

/// Get an index of Refresh Tokens, returning a vector of Refresh Tokens
///
/// # Parameters
///
/// * `limit` - An i64 limiting the page length
/// * `offset` - An i64 of where the limit should start
/// * `database` - An sqlx database pool that the refresh tokens will be searched in.
/// ---
#[tracing::instrument(
	name = "Index of Refresh Tokens with offset and limit"
	skip(database)
)]
pub async fn select_refresh_token_index(
	limit: &i64,
	offset: &i64,
	database: &sqlx::Pool<sqlx::Postgres>,
) -> Result<Vec<RefreshTokenModel>, BackendError> {
	let refresh_token_records = sqlx::query_as!(
		RefreshTokenModel,
		r#"
			SELECT * 
			FROM refresh_tokens 
			ORDER BY id
			LIMIT $1 OFFSET $2
		"#,
		&limit,
		&offset,
	)
	.fetch_all(database)
	.await?;

	tracing::debug!("Refresh Token records returned from database: {refresh_token_records:#?}");

	Ok(refresh_token_records)
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

	// Test getting Refresh Token from database using unique UUID
	#[sqlx::test]
	async fn get_refresh_token_record_by_id(database: Pool<Postgres>) -> Result<()> {
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
		let database_record = select_refresh_token_by_id(&refresh_token.id, &database).await?;
		// println!("{record:#?}");

		//-- Checks (Assertions)
		assert_eq!(database_record.id, refresh_token.id);
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

	// Test getting Refresh Token from database using user id
	#[sqlx::test]
	async fn get_user_records_by_user_id(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate random user
		let random_user = generate_random_user()?;

		// Insert random user into the database
		insert_user(&random_user, &database).await?;

		// Generate a random number of Refresh Tokens
		let random_count: i64 = (10..30).fake::<i64>();

		// Initiate a new vector of Refresh Token
		let mut test_vec: Vec<RefreshTokenModel> = Vec::new();

		// Iterate through the random number
		for _count in 0..random_count {
			// Generate refresh token
			let refresh_token = generate_random_refresh_token(random_user.id)?;

			// Insert refresh token in the database and push result to vector
			test_vec.push(insert_refresh_token(&refresh_token, &database).await?);
		}

		//-- Execute Function (Act)
		// Get the Refresh Tokens
		let database_record = select_refresh_token_by_user_id(&random_user.id, &database).await?;
		// println!("{record:#?}");

		//-- Checks (Assertions)
		// Check the number of database records equals the random number added to the database
		assert_eq!(database_record.len() as i64, random_count);

		// Generate a radom number from the random count to check equal
		let random_vec_index = (0..random_count).fake::<i64>() as usize;

		let random_token = &database_record[random_vec_index];
		// println!("{random_token:#?}");

		// Check Refresh Tokens are equal
		assert_eq!(
			database_record[random_vec_index].id,
			test_vec[random_vec_index].id
		);

		assert_eq!(
			database_record[random_vec_index].user_id,
			test_vec[random_vec_index].user_id
		);

		assert_eq!(
			database_record[random_vec_index].refresh_token,
			test_vec[random_vec_index].refresh_token
		);

		assert_eq!(
			database_record[random_vec_index].is_active,
			test_vec[random_vec_index].is_active
		);

		assert_eq!(
			database_record[random_vec_index].created_on.timestamp(),
			test_vec[random_vec_index].created_on.timestamp()
		);

		// -- Return
		Ok(())
	}

	// Test Req7est Token query
	#[sqlx::test]
	async fn get_request_token_index(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate random user
		let random_user = generate_random_user()?;

		// Insert random user into the database
		insert_user(&random_user, &database).await?;

		// Generate a random number of Refresh Tokens
		let random_count: i64 = (10..30).fake::<i64>();

		// Initiate a new vector of Refresh Token
		let mut test_vec: Vec<RefreshTokenModel> = Vec::new();

		// Iterate through the random number
		for _count in 0..random_count {
			// Generate refresh token
			let refresh_token = generate_random_refresh_token(random_user.id)?;

			// Insert refresh token in the database and push result to vector
			test_vec.push(insert_refresh_token(&refresh_token, &database).await?);
		}

		//-- Execute Function (Act)
		// Get a random limit number
		let random_limit = (1..random_count).fake::<i64>();

		// Get a random offset number
		let random_offset = (1..random_count).fake::<i64>();

		let database_records =
			select_refresh_token_index(&random_limit, &random_offset, &database).await?;


		//-- Checks (Assertions)
		// Check for edge case when limit is restricted by not being enough entries
		let count_less_offset: i64 = random_count - random_offset;
		let expected_records = if count_less_offset < random_limit {
			count_less_offset
		} else {
			random_limit
		};

		assert_eq!(database_records.len() as i64, expected_records);

		//-- Return
		Ok(())
	}
}
