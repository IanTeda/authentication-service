//-- ./src/database/refresh_tokens/update.rs

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

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::database::RefreshTokens;
use crate::prelude::BackendError;

impl RefreshTokens {
    /// Update a `Refresh Token` in the database, returning a result with a RefreshTokenModel instance
    /// or Sqlx error.
    ///
    /// # Parameters
    ///
    /// * `self` - A RefreshTokenModel instance.
    /// * `database` - An Sqlx database connection pool.
    /// ---
    #[tracing::instrument(
        name = "Update a Refresh Token in the database: ",
        skip(self, database),
        fields(
            db_id = % self.id,
            user_id = % self.user_id,
            refresh_token = % self.token.as_ref(),
            is_active = % self.is_active,
            created_on = % self.created_on,
        ),
    )]
    pub async fn update(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<RefreshTokens, BackendError> {
        let database_record = sqlx::query_as!(
            RefreshTokens,
            r#"
				UPDATE refresh_tokens 
				SET user_id = $2, token = $3, is_active = $4
				WHERE id = $1 
				RETURNING *
			"#,
            self.id,
            self.user_id,
            self.token.as_ref(),
            self.is_active,
        )
        .fetch_one(database)
        .await?;

        tracing::debug!(
            "Refresh Token database records retrieved: {database_record:#?}"
        );

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
        name = "Revoke a Refresh Token in the database: ",
        skip(self, database),
        fields(
            db_id = % self.id,
            user_id = % self.user_id,
            refresh_token = % self.token.as_ref(),
            is_active = % self.is_active,
            created_on = % self.created_on,
        ),
    )]
    pub async fn revoke(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<Self, BackendError> {
        let database_record = sqlx::query_as!(
            RefreshTokens,
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

        tracing::debug!(
            "Refresh Token database records retrieved: {database_record:#?}"
        );

        Ok(database_record)
    }

    /// Revoke (make non-active) all Refresh Token in the database associated to
    /// the user_id, returning a result with the number RefreshTokenModels revoked
    /// or an SQLx error
    ///
    /// # Parameters
    ///
    /// * `self` - A RefreshTokenModel instance for the user_id Refresh Tokens to be revoked.
    /// * `database` - An Sqlx database connection pool.
    /// ---
    #[tracing::instrument(
        name = "Revoke all Refresh Tokens for a given user_id in the database: ",
        skip(self, database)
    )]
    pub async fn revoke_all_associated(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query_as!(
            RefreshTokens,
            r#"
                    UPDATE refresh_tokens
                    SET is_active = false
                    WHERE user_id = $1
                "#,
            self.user_id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!(
            "Refresh Token database records updated: {rows_affected:#?}"
        );

        Ok(rows_affected)
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
        name = "Revoke all Refresh Tokens for a given user_id in the database: ",
        skip(database)
    )]
    pub async fn revoke_user_id(
        user_id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query_as!(
            RefreshTokens,
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

        tracing::debug!(
            "Refresh Token database records updated: {rows_affected:#?}"
        );

        Ok(rows_affected)
    }
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {
    use fake::Fake;
    use sqlx::{Pool, Postgres};

    use crate::database;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    #[sqlx::test]
    async fn update_refresh_token(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        // Generate refresh token
        let mut refresh_token =
            database::RefreshTokens::mock_data(&random_user).await?;

        // Insert refresh token in the database for deleting
        refresh_token.insert(&database).await?;

        // Generate random user for testing
        let random_user_update = database::Users::mock_data()?;

        // Insert user in the database
        random_user_update.insert(&database).await?;

        // Generate refresh token updates
        let refresh_token_update =
            database::RefreshTokens::mock_data(&random_user_update).await?;

        // Update Refresh Token data
        refresh_token.user_id = refresh_token_update.user_id;
        refresh_token.token = refresh_token_update.token;
        refresh_token.is_active = refresh_token_update.is_active;

        //-- Execute Function (Act)
        // Generate an updated Refresh Token
        let database_record = refresh_token.update(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(database_record, refresh_token);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn revoke_refresh_token(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        // Generate refresh token
        let mut refresh_token =
            database::RefreshTokens::mock_data(&random_user).await?;

        // Set Refresh Token active to true
        refresh_token.is_active = true;

        // Insert Refresh Token into database
        refresh_token.insert(&database).await?;

        // Insert refresh token in the database for deleting
        refresh_token.revoke(&database).await?;

        //-- Execute Function (Act)
        // Generate an updated Refresh Token
        let database_record =
            database::RefreshTokens::from_id(&refresh_token.id, &database).await?;

        //-- Checks (Assertions)
        assert_eq!(database_record.is_active, false);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn revoke_all_user_refresh_tokens(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        let mut test_vec: Vec<database::RefreshTokens> = Vec::new();
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate refresh token
            let mut refresh_token =
                database::RefreshTokens::mock_data(&random_user).await?;

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
            database::RefreshTokens::revoke_user_id(&random_user.id, &database)
                .await?;

        //-- Checks (Assertions)
        assert_eq!(rows_affected, random_count as u64);

        // Pick a random Refresh Token to asset is_active as false
        let random_index_number = (1..random_count).fake::<i64>() as usize;

        // Get the database record
        let database_record = database::RefreshTokens::from_id(
            &test_vec[random_index_number].id,
            &database,
        )
        .await?;

        // Check the is_active status is false
        assert_eq!(database_record.is_active, false);

        // -- Return
        Ok(())
    }
}
