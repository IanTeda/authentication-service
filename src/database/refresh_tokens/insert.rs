//-- ./src/database/refresh_tokens/insert.rs

//! Create [insert] a Refresh Token into the database, returning a Result with a
//! RefreshTokenModel instance
//! ---

// #![allow(unused)] // For development only

use sqlx::{Pool, Postgres};

use crate::prelude::*;

use super::model::RefreshTokens;

impl RefreshTokens {
    /// Insert a Refresh Token into the database, returning the database record
    /// as a RefreshTokenModel.
    ///
    /// # Parameters
    ///
    /// * `refresh_tokens` - A Refresh Token instance
    /// * `database` - An Sqlx database connection pool
    /// ---
    #[tracing::instrument(
        name = "Insert a new Refresh Token into the database: ",
        skip(self, database),
        fields(
            id = % self.id,
            user_id = % self.user_id,
            refresh_tokens = % self.token.as_ref(),
            is_active = % self.is_active,
            created_on = % self.created_on,
        ),
    )]
    pub async fn insert(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<Self, BackendError> {
        let database_record = sqlx::query_as!(
            RefreshTokens,
            r#"
				INSERT INTO refresh_tokens (id, user_id, token, is_active, created_on)
				VALUES ($1, $2, $3, $4, $5) 
				RETURNING *
			"#,
            self.id,
            self.user_id,
            self.token.as_ref(),
            self.is_active,
            self.created_on
        )
        .fetch_one(database)
        .await?;

        tracing::debug!(
            "Refresh Token database records retrieved: {database_record:#?}"
        );

        Ok(database_record)
    }
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {
    use sqlx::{Pool, Postgres};

    use crate::database;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    // Test inserting into database
    #[cfg(feature = "mocks")]
    #[sqlx::test]
    async fn create_database_record(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        // Generate refresh token
        let random_refresh_token =
            database::RefreshTokens::mock_data(&random_user.id).await?;

        //-- Execute Function (Act)
        // Insert refresh token into database
        let database_record = random_refresh_token.insert(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(database_record, random_refresh_token);

        // -- Return
        Ok(())
    }
}
