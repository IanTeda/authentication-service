//-- ./src/database/refresh_token/insert.rs

//! Create [insert] a Refresh Token into the database, returning a Result with a
//! RefreshTokenModel instance
//! ---

// #![allow(unused)] // For development only

use sqlx::{Pool, Postgres};

use crate::prelude::*;

use super::model::RefreshTokenModel;

impl super::RefreshTokenModel {
	/// Insert a Refresh Token into the database, returning the database record
	/// as a RefreshTokenModel.
	///
	/// # Parameters
	///
	/// * `refresh_token` - A Refresh Token instance
	/// * `database` - An Sqlx database connection pool
	/// ---
	#[tracing::instrument(
		name = "Insert a new Refresh Token into the database."
		skip(self, database)
		fields(
        	db_id = %self.id,
			// user_id = %self.user_id,
			// refresh_token = %self.refresh_token.as_ref(),
			// is_active = %self.is_active,
			// created_on = %self.created_on,
    	)
	)]
	pub async fn insert(
		&self,
		database: &Pool<Postgres>,
	) -> Result<Self, BackendError> {
		let database_record = sqlx::query_as!(
			RefreshTokenModel,
			r#"
				INSERT INTO refresh_tokens (id, user_id, refresh_token, is_active, created_on) 
				VALUES ($1, $2, $3, $4, $5) 
				RETURNING *
			"#,
			self.id,
			self.user_id,
			self.refresh_token.as_ref(),
			self.is_active,
			self.created_on
		)
		.fetch_one(database)
		.await?;

		Ok(database_record)
	}
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	use crate::database::UserModel;

	// Bring module functions into test scope
	use super::*;

	use sqlx::{Pool, Postgres};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	// Test inserting into database
	#[sqlx::test]
	async fn create_database_record(database: Pool<Postgres>) -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate radom user for testing
		let random_user = UserModel::mock_data().await?;

		// Insert user in the database
		random_user.insert(&database).await?;

		// Generate refresh token
		let refresh_token =
			RefreshTokenModel::mock_data(&random_user.id).await?;

		//-- Execute Function (Act)
		// Insert refresh token into database
		refresh_token.insert(&database).await?;

		// Get database record that was insert above, to be sure.
		let database_record = RefreshTokenModel::from_id(&refresh_token.id, &database).await?;

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
}
