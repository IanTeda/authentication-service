//-- ./src/database/users/delete.rs

//! Delete User in the database, returning a Result with a boolean
//! ---

// #![allow(unused)] // For development only

use crate::{
	database::users::UserModel, domains::EmailAddress, prelude::*
};

use uuid::Uuid;

/// Delete a User from the database by querying the User uuid, returning a Result
/// with a boolean or an sqlx error.
///
/// # Parameters
///
/// * `id` - The uuid of thing to be returned
/// * `database` - An sqlx database pool that the thing will be searched in.
#[tracing::instrument(
	name = "Delete a User from the database using its id (uuid)."
	skip(id, database)
)]
pub async fn delete_user_by_id(
	id: &Uuid,
	database: &sqlx::Pool<sqlx::Postgres>,
) -> Result<bool, BackendError> {

	let rows_affected = sqlx::query!(
		r#"
			Delete 
			FROM users 
			WHERE id = $1
		"#,
		id
	)
	.execute(database)
	.await?
    .rows_affected();

    let confirm_deleted: bool = rows_affected != 0;
	
    tracing::debug!("User record retrieved form database: {rows_affected:#?}");
    
	Ok(confirm_deleted)
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
	async fn delete_user_record_by_id(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_test_user = generate_random_user()?;
        insert_user(&random_test_user, &database).await?;
		// println!("{test_thing:#?}");

		//-- Execute Function (Act)
		// Insert user into database
		let database_user = delete_user_by_id(&random_test_user.id, &database).await?;
		// println!("{record:#?}");

        //-- Checks (Assertions)
		assert!(database_user);

		// -- Return
		Ok(())
	}

        // Test getting user from database using unique UUID
	#[sqlx::test]
	async fn delete_user_false(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_test_user = generate_random_user()?;
        insert_user(&random_test_user, &database).await?;
        let random_user_id = generate_random_user()?.id;
		// println!("{test_thing:#?}");

		//-- Execute Function (Act)
		// Insert user into database
		let database_user = delete_user_by_id(&random_user_id, &database).await?;
		// println!("{record:#?}");

        //-- Checks (Assertions)
		assert!(!database_user);

		// -- Return
		Ok(())
	}
}