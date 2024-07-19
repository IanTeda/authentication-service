//-- ./src/database/refresh_tokens/update.rs

// #![allow(unused)] // For development only

//! Update Refresh Tokens in the database
//!
//! This module has six impl functions for RefreshTokenModel:
//!
//!  1. `update` - Self: Update self in the database
//!  2. `revoke` - Self: Revoke (is_active = false) the RefreshTokenModel instance in the database
//!  3. `revoke_by_id` - Uuid: Revoke (is_active = false) for a give row PK (id) 
//!  4. `revoke_associated` - Self: Revoke (is_active = false) all tokens associated with user_id in self
//!  5. `revoke_user_id` - Uuid: Revoke (is_active = false) all the rows in the database for the user_id
//!  6. `revoke_all` - Revoke (is_active = false) all database entries
//! ---

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::database::RefreshTokens;
use crate::prelude::BackendError;

impl RefreshTokens {
    /// Update a self in the database, returning a result with a RefreshTokenModel instance
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

    /// Revoke (make non-active) self in the database, returning a result
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
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query_as!(
            RefreshTokens,
            r#"
                    UPDATE refresh_tokens
                    SET is_active = false
                    WHERE id = $1
                "#,
            self.id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!(
            "Refresh Token database records updated: {rows_affected:#?}"
        );

        Ok(rows_affected)
    }

    /// Revoke (make non-active) a Refresh Token with a give PK (id) in the database, 
    /// returning a result with the number of rows affected or and SQLx error
    ///
    /// # Parameters
    ///
    /// * `id` - Uuid: The database row PK (id).
    /// * `database` - An Sqlx database connection pool.
    /// ---
    #[tracing::instrument(
        name = "Revoke a Refresh Token in the database by id: ",
        skip(id, database),
        fields(
            db_id = %id,
        ),
    )]
    pub async fn revoke_by_id(
        id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query_as!(
            RefreshTokens,
            r#"
                    UPDATE refresh_tokens
                    SET is_active = false
                    WHERE id = $1
                "#,
            id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!(
            "Refresh Token database records updated: {rows_affected:#?}"
        );

        Ok(rows_affected)
    }

    /// Revoke (make non-active) all Refresh Token in the database associated to
    /// the self (user_id), returning a result with the number of rows revoked
    /// or an SQLx error
    ///
    /// # Parameters
    ///
    /// * `self` - RefreshToken: Self with the user_id to revoke.
    /// * `database` - An Sqlx database connection pool.
    /// ---
    #[tracing::instrument(
        name = "Revoke all Refresh Tokens associated with Self user_id: ",
        skip(self, database),
        fields(
            db_id = % self.id,
            user_id = % self.user_id,
            refresh_token = % self.token.as_ref(),
            is_active = % self.is_active,
            created_on = % self.created_on,
        ),
    )]
    pub async fn revoke_associated(
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
        skip(database),
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


    /// Revoke (make non-active) all Refresh Token in the database, returning a 
    /// result with the number of rows revoked or an SQLx error.
    ///
    /// # Parameters
    ///
    /// * `database` - An Sqlx database connection pool.
    /// ---
    #[tracing::instrument(
        name = "Revoke all Refresh Tokens in the database: ",
        skip(database)
    )]
    pub async fn revoke_all(
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query_as!(
            RefreshTokens,
            r#"
                UPDATE refresh_tokens
                SET is_active = false
            "#,
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!(
            "All Refresh Token revoked in the database, records updated: {rows_affected:#?}"
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
    async fn update_self(database: Pool<Postgres>) -> Result<()> {
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
    async fn revoke_self(database: Pool<Postgres>) -> Result<()> {
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
    async fn revoke_by_id(database: Pool<Postgres>) -> Result<()> {
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
        let _database_record = refresh_token.insert(&database).await?;


        // Insert refresh token in the database for deleting
        // refresh_token.revoke(&database).await?;

        //-- Execute Function (Act)
        //Revoke tokens
        // let database_record =
        //     database::RefreshTokens::from_id(&refresh_token.id, &database).await?;
        let rows_affected = database::RefreshTokens::revoke_by_id(&refresh_token.id, &database).await?;

        //-- Checks (Assertions)
        // The one entry should be affected
        assert_eq!(rows_affected, 1);

        // Get record from the database
        let database_record = database::RefreshTokens::from_id(&refresh_token.id, &database).await?;

        // Check the is_active status is false
        assert_eq!(database_record.is_active, false);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn revoke_self_associated(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        let mut refresh_token =
                database::RefreshTokens::mock_data(&random_user).await?;

        // Set Refresh Token active to true
        refresh_token.is_active = true;

        // Insert refresh token in the database for deleting
        refresh_token.insert(&database).await?;

        let mut test_vec: Vec<database::RefreshTokens> = Vec::new();
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate refresh token
            let mut loop_refresh_token =
                database::RefreshTokens::mock_data(&random_user).await?;

            // Set Refresh Token active to true
            loop_refresh_token.is_active = true;

            // Insert refresh token in the database for deleting
            loop_refresh_token.insert(&database).await?;

            // Add Refresh Token to a Vec
            test_vec.push(loop_refresh_token);
        }

        //-- Execute Function (Act)
        // Generate an updated Refresh Token
        let rows_affected = refresh_token.revoke_associated(&database).await?;

        //-- Checks (Assertions)
        // There is an extra row outside the loop so we can use it for association
        assert_eq!(rows_affected, random_count as u64 + 1);

        // Get record from the database
        let database_record = database::RefreshTokens::from_id(&refresh_token.id, &database).await?;

        // Check the is_active status is false
        assert_eq!(database_record.is_active, false);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn revoke_all_user_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        let mut refresh_token =
                database::RefreshTokens::mock_data(&random_user).await?;

        // Set Refresh Token active to true
        refresh_token.is_active = true;

        // Insert refresh token in the database for deleting
        refresh_token.insert(&database).await?;

        let mut test_vec: Vec<database::RefreshTokens> = Vec::new();
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate refresh token
            let mut loop_refresh_token =
                database::RefreshTokens::mock_data(&random_user).await?;

            // Set Refresh Token active to true
            loop_refresh_token.is_active = true;

            // Insert refresh token in the database for deleting
            loop_refresh_token.insert(&database).await?;

            // Add Refresh Token to a Vec
            test_vec.push(loop_refresh_token);
        }

        //-- Execute Function (Act)
        // Generate an updated Refresh Token
        let rows_affected = database::RefreshTokens::revoke_user_id(&refresh_token.user_id, &database).await?;

        //-- Checks (Assertions)
        // There is an extra row outside the loop so we can use it for association
        assert_eq!(rows_affected, random_count as u64 + 1);

        // Get record from the database
        let database_record = database::RefreshTokens::from_id(&refresh_token.id, &database).await?;

        // Check the is_active status is false
        assert_eq!(database_record.is_active, false);

        // -- Return
        Ok(())
    }


    #[sqlx::test]
    async fn revoke_all_red_button(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        let mut refresh_token =
                database::RefreshTokens::mock_data(&random_user).await?;

        // Set Refresh Token active to true
        refresh_token.is_active = true;

        // Insert refresh token in the database for deleting
        refresh_token.insert(&database).await?;

        let mut test_vec: Vec<database::RefreshTokens> = Vec::new();
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate refresh token
            let mut loop_refresh_token =
                database::RefreshTokens::mock_data(&random_user).await?;

            // Set Refresh Token active to true
            loop_refresh_token.is_active = true;

            // Insert refresh token in the database for deleting
            loop_refresh_token.insert(&database).await?;

            // Add Refresh Token to a Vec
            test_vec.push(loop_refresh_token);
        }

        //-- Execute Function (Act)
        // Generate an updated Refresh Token
        let rows_affected = database::RefreshTokens::revoke_all(&database).await?;

        //-- Checks (Assertions)
        // There is an extra row outside the loop so we can use it for association
        assert_eq!(rows_affected, random_count as u64 + 1);

        // Get record from the database
        let database_record = database::RefreshTokens::from_id(&refresh_token.id, &database).await?;

        // Check the is_active status is false
        assert_eq!(database_record.is_active, false);

        // -- Return
        Ok(())
    }
}
