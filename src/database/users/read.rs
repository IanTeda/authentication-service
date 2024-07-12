//-- ./src/database/users/read.rs

//! Read User[s] int the database, returning a Result with a UserModel instance
//! or a Vec[UserModel]
//! ---

// #![allow(unused)] // For development only

use crate::{database::users::UserModel, domains::EmailAddress, prelude::*};

use uuid::Uuid;

impl super::UserModel {
	/// Get a User from the database by querying the User uuid, returning a User Model (Self)
	/// instance or sqlx error.
	///
	/// # Parameters
	///
	/// * `id` - The uuid of thing to be returned
	/// * `database` - An sqlx database pool that the thing will be searched in.
	/// ---
	#[tracing::instrument(
		name = "Read a User from the database: "
		skip(database)
	)]
	pub async fn from_user_id(
		id: &Uuid,
		database: &sqlx::Pool<sqlx::Postgres>,
	) -> Result<Self, BackendError> {
		let database_record = sqlx::query_as!(
			UserModel,
			r#"
				SELECT * 
				FROM users 
				WHERE id = $1
			"#,
			id
		)
		.fetch_one(database)
		.await?;

		Ok(database_record)
	}

	/// Get User from the database by querying the User Email, returning a User
	/// Model (Self) instance or sqlx error.
	///
	/// # Parameters
	///
	/// * `id` - The uuid of thing to be returned
	/// * `database` - An sqlx database pool that the thing will be searched in.
	/// ---
	#[tracing::instrument(
		name = "Read a User from the database: "
		skip(email, database)
		fields(
        	user_email = %email.as_ref(),
    	)
	)]
	pub async fn from_user_email(
		email: &EmailAddress,
		database: &sqlx::Pool<sqlx::Postgres>,
	) -> Result<Self, BackendError> {
		let database_record = sqlx::query_as!(
			UserModel,
			r#"
				SELECT * 
				FROM users 
				WHERE email = $1
			"#,
			email.as_ref()
		)
		.fetch_one(database)
		.await?;

		tracing::debug!("User database record retrieved: {database_record:#?}");

		Ok(database_record)
	}

	/// Get an index of Users, returning a vector of Users
	///
	/// # Parameters
	///
	/// * `limit` - An i64 limiting the page length
	/// * `offset` - An i64 of where the limit should start
	/// * `database` - An sqlx database pool that the things will be searched in.
	/// ---
	#[tracing::instrument(
		name = "Index of Users with offset and limit"
		skip(database)
	)]
	pub async fn index(
		limit: &i64,
		offset: &i64,
		database: &sqlx::Pool<sqlx::Postgres>,
	) -> Result<Vec<UserModel>, BackendError> {
		let database_records = sqlx::query_as!(
			UserModel,
			r#"
				SELECT * 
				FROM users 
				ORDER BY id
				LIMIT $1 OFFSET $2
			"#,
			limit,
			offset,
		)
		.fetch_all(database)
		.await?;

		Ok(database_records)
	}
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	use crate::database;

	// Bring module functions into test scope
	use super::*;

	use fake::Fake;
	use sqlx::{Pool, Postgres};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	// Test getting user from database using unique UUID
	#[sqlx::test]
	async fn get_user_record_by_id(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_test_user = UserModel::mock_data().await?;

		// Insert user in the database
		random_test_user.insert(&database).await?;

		//-- Execute Function (Act)
		// Retrieve User from database
		let database_record =
			database::UserModel::from_user_id(&random_test_user.id, &database)
				.await?;

		//-- Checks (Assertions)
		assert_eq!(database_record.id, random_test_user.id);
		assert_eq!(database_record.email, random_test_user.email);
		assert_eq!(database_record.user_name, random_test_user.user_name);
		assert_eq!(
			database_record.password_hash,
			random_test_user.password_hash
		);
		assert_eq!(database_record.is_active, random_test_user.is_active);
		// Use timestamp because Postgres time precision is less than Rust
		assert_eq!(
			database_record.created_on.timestamp(),
			random_test_user.created_on.timestamp()
		);

		// -- Return
		Ok(())
	}

	// Test getting user from database using unique email
	#[sqlx::test]
	async fn get_user_record_by_email(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_test_user = UserModel::mock_data().await?;

		// Insert user in the database
		random_test_user.insert(&database).await?;

		//-- Execute Function (Act)
		// Insert user into database
		let database_user =
			database::UserModel::from_user_email(&random_test_user.email, &database)
				.await?;
		// println!("{record:#?}");

		//-- Checks (Assertions)
		assert_eq!(database_user.id, random_test_user.id);
		assert_eq!(database_user.email, random_test_user.email);
		assert_eq!(database_user.user_name, random_test_user.user_name);
		assert_eq!(database_user.password_hash, random_test_user.password_hash);
		assert_eq!(database_user.is_active, random_test_user.is_active);
		// Use timestamp because Postgres time precision is less than Rust
		assert_eq!(
			database_user.created_on.timestamp(),
			random_test_user.created_on.timestamp()
		);

		// -- Return
		Ok(())
	}

	// Test thing query
	#[sqlx::test]
	async fn get_users_in_database(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		let random_count: i64 = (10..30).fake::<i64>();
		let mut test_vec: Vec<UserModel> = Vec::new();
		for _count in 0..random_count {
			// Generate radom user for testing
			let random_test_user = UserModel::mock_data().await?;

			// Insert user in the database
			test_vec.push(random_test_user.insert(&database).await?);
		}

		//-- Execute Function (Act)
		let random_limit = (1..random_count).fake::<i64>();
		let random_offset = (1..random_count).fake::<i64>();
		let records =
			database::UserModel::index(&random_limit, &random_offset, &database)
				.await?;

		//-- Checks (Assertions)
		let count_less_offset: i64 = random_count - random_offset;

		let expected_records = if count_less_offset < random_limit {
			count_less_offset
		} else {
			random_limit
		};

		// let random_vec_index: i64 = (1..expected_records).fake::<i64>() - 1;
		// let random_test_vec_index = random_offset + random_vec_index;
		// let random_record_thing = &records[random_vec_index as usize];
		// let random_test_thing = &test_vec[random_test_vec_index as usize];

		assert_eq!(records.len() as i64, expected_records);
		// assert_eq!(random_record_thing.id, random_test_thing.id);

		Ok(())
	}
}
