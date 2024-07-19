//-- ./src/database/refresh_tokens/delete.rs

//! Delete RefreshToken in the database, returning a Result with an u64 of the number
//! of rows affected.
//!
//! This module adds two impl methods to RefreshTokenModel:
//!
//! 1. `delete`: Delete a Refresh Token instance in the database, returning the number of rows affected.
//! 2. `delete_by_id`: Delete a Refresh Token by row PK (id)
//! 3. `delete_all_user_id`: Delete all Refresh Token instances for a given user_id, returning the number of rows affected
//! 4. `delete_all`: Delete all Refresh Tokens in the database, returning the number of rows deleted
//! ---

// #![allow(unused)] // For development only

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::database::RefreshTokens;
use crate::prelude::*;

impl RefreshTokens {
    /// Delete a Refresh Token from the database, returning a Result with the number 
    /// of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `self` - The Refresh Token instance to be deleted.
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Delete a Refresh Token from the database using it self: ",
        skip(self, database),
        fields(
            db_id = % self.id,
            user_id = % self.user_id,
            refresh_token = % self.token.as_ref(),
            is_active = % self.is_active,
            created_on = % self.created_on,
        ),
    )]
    pub async fn delete(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
                    Delete
                    FROM refresh_tokens
                    WHERE id = $1
                "#,
            self.id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!(
            "Refresh Token database records deleted: {rows_affected:#?}"
        );

        Ok(rows_affected)
    }

    /// Delete a Refresh Token from the database by querying the Refresh Token uuid, 
    /// returning a Result with the number of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `id` - Uuid: The Refresh Token instance to be deleted.
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Delete a Refresh Token from the database using its id (uuid): ",
        skip(id, database),
        fields(
            db_id = %id,
        ),
    )]
    pub async fn delete_by_id(
        id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
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

        tracing::debug!(
            "Refresh Token database records deleted: {rows_affected:#?}"
        );

        Ok(rows_affected)
    }

    /// Delete all Refresh Tokens from the database associated with a user_id, 
    /// returning a Result with the number of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `user_id` - Uuid: The user_id for the Refresh Tokens to be deleted
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Delete all Refresh Token from the database associated with user_id (uuid): ",
        skip(user_id, database),
        fields(
            user_id = % user_id,
        ),
    )]
    pub async fn delete_all_user_id(
        user_id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
                    Delete
                    FROM refresh_tokens
                    WHERE user_id = $1
                "#,
            user_id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!(
            "Refresh Token database records deleted: {rows_affected:#?}"
        );

        Ok(rows_affected)
    }

    /// Delete all Refresh Tokens from the database, returning a Result
    /// with the u64 number of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `user_id` - The user_id for the Refresh Tokens to be deleted
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Delete all Refresh Token from the database: ",
        skip(database),
    )]
    pub async fn delete_all(
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
                Delete
                FROM refresh_tokens

            "#,

        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!(
            "Refresh Token database records deleted: {rows_affected:#?}"
        );

        Ok(rows_affected)
    }
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

    // Bring module functions into test scope
    // use super::*;

    use fake::Fake;
    use sqlx::{Pool, Postgres};

    use crate::database;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn delete_self(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        // Generate refresh token
        let refresh_token =
            database::RefreshTokens::mock_data(&random_user).await?;

        // Insert refresh token in the database for deleting
        refresh_token.insert(&database).await?;

        //-- Execute Function (Act)
        // Delete self from the database
        let rows_affected = refresh_token.delete(&database).await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 1);

        // -- Return
        Ok(())
    }

      // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn delete_by_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        // Generate refresh token
        let refresh_token =
            database::RefreshTokens::mock_data(&random_user).await?;

        // Insert refresh token in the database for deleting
        refresh_token.insert(&database).await?;

        //-- Execute Function (Act)
        // Delete database row
        let rows_affected = database::RefreshTokens::delete_by_id(&refresh_token.id, &database).await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        // Each id is unique to the row, so I should equal 1
        assert_eq!(rows_affected, 1);

        // -- Return
        Ok(())
    }

    // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn delete_user_id_associated(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        // Add a random number of refresh tokens for the given user
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate refresh token
            let refresh_token =
                database::RefreshTokens::mock_data(&random_user).await?;

            // Insert refresh token in the database for deleting
            refresh_token.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Delete all database entries for the random user id
        let rows_affected =
            database::RefreshTokens::delete_all_user_id(&random_user.id, &database)
                .await?;
        // println!("{rows_affected:#?}");

        //-- Checks (Assertions)
        // Rows affected should equal the random number of rows inserted
        assert_eq!(rows_affected, random_count as u64);

        // -- Return
        Ok(())
    }

        // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn delete_all(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate a random number of Refresh Tokens
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate random user for testing
            let random_user = database::Users::mock_data()?;

            // Insert user in the database
            random_user.insert(&database).await?;

            // Generate refresh token
            let refresh_token =
                database::RefreshTokens::mock_data(&random_user).await?;

            // Insert refresh token in the database for deleting
            refresh_token.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Delete all rows in the database table
        let rows_affected =
            database::RefreshTokens::delete_all(&database)
                .await?;

        //-- Checks (Assertions)
        // Rows affected should equal the number of random entries
        assert_eq!(rows_affected, random_count as u64);

        // -- Return
        Ok(())
    }
}
