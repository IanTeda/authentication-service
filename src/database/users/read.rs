//-- ./src/database/users/read.rs

//! Read User[s] int the database, returning a Result with a UserModel instance
//! or a Vec[UserModel]
//! ---

// #![allow(unused)] // For development only

use crate::{database::users::UserModel, domains::EmailAddress, prelude::*};

use uuid::Uuid;

/// Get User from the database by querying the User uuid,
/// returning a thing instance or sqlx error.
///
/// # Parameters
///
/// * `id` - The uuid of thing to be returned
/// * `database` - An sqlx database pool that the thing will be searched in.
#[tracing::instrument(
	name = "Get a User from the database using its id (uuid)."
	skip(id, database)
)]
pub async fn select_user_by_id(
	id: &Uuid,
	database: &sqlx::Pool<sqlx::Postgres>,
) -> Result<UserModel, BackendError> {
	let user_record = sqlx::query_as!(
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

	tracing::debug!("User record retrieved form database: {user_record:#?}");

	Ok(user_record)
}

/// Get User from the database by querying the thing uuid,
/// returning a thing instance or sqlx error.
///
/// # Parameters
///
/// * `id` - The uuid of thing to be returned
/// * `database` - An sqlx database pool that the thing will be searched in.
#[tracing::instrument(
	name = "Get a User from the database using its id (uuid)."
	skip(email, database)
)]
pub async fn select_user_by_email(
	email: &EmailAddress,
	database: &sqlx::Pool<sqlx::Postgres>,
) -> Result<UserModel, BackendError> {
	let user_record = sqlx::query_as!(
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

	tracing::debug!("User record retrieved form database: {user_record:#?}");

	Ok(user_record)
}

/// Get an index of things, returning a vector of Things
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
pub async fn select_user_index(
	limit: &i64,
	offset: &i64,
	database: &sqlx::Pool<sqlx::Postgres>,
) -> Result<Vec<UserModel>, BackendError> {
	let users_index = sqlx::query_as!(
		UserModel,
		r#"
			SELECT * 
			FROM users 
			ORDER BY id
			LIMIT $1 OFFSET $2
		"#,
		&limit,
		&offset,
	)
	.fetch_all(database)
	.await?;

	tracing::debug!("Database records returned from database: {users_index:#?}");

	Ok(users_index)
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	// Bring module functions into test scope
	use super::*;

	use crate::database::users::{insert_user, model::tests::generate_random_user};

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
		let random_test_user = generate_random_user()?;
		insert_user(&random_test_user, &database).await?;
		// println!("{test_thing:#?}");

		//-- Execute Function (Act)
		// Insert user into database
		let database_user = select_user_by_id(&random_test_user.id, &database).await?;
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

	// Test getting user from database using unique email
	#[sqlx::test]
	async fn get_user_record_by_email(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_test_user = generate_random_user()?;
		insert_user(&random_test_user, &database).await?;
		// println!("{test_thing:#?}");

		//-- Execute Function (Act)
		// Insert user into database
		let database_user = select_user_by_email(&random_test_user.email, &database).await?;
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
	async fn get_users_in_database(pool: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		let random_count: i64 = (10..30).fake::<i64>();
		let mut test_vec: Vec<UserModel> = Vec::new();
		for _count in 0..random_count {
			let test_user = generate_random_user()?;
			test_vec.push(insert_user(&test_user, &pool).await?);
		}

		//-- Execute Function (Act)
		let random_limit = (1..random_count).fake::<i64>();
		let random_offset = (1..random_count).fake::<i64>();
		let records = select_user_index(&random_limit, &random_offset, &pool).await?;

		//-- Checks (Assertions)
		let count_less_offset: i64 = random_count - random_offset;

		let expected_records = if count_less_offset < random_limit {
			count_less_offset
		} else {
			random_limit
		};

		let random_vec_index: i64 = (1..expected_records).fake::<i64>() - 1;
		let random_test_vec_index = random_offset + random_vec_index;
		let random_record_thing = &records[random_vec_index as usize];
		let random_test_thing = &test_vec[random_test_vec_index as usize];

		assert_eq!(records.len() as i64, expected_records);
		// assert_eq!(random_record_thing.id, random_test_thing.id);

		Ok(())
	}
}
