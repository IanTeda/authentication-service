//-- ./src/database/refresh_tokens/read.rs

//! Read RefreshToken[s] in the database, returning a Result with a RefreshTokenModel instance
//! or a Vec[RefreshTokenModel]
//! ---

// // #![allow(unused)] // For development only

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::database::RefreshTokens;
use crate::prelude::*;

impl RefreshTokens {
    /// Get a Refresh Token from the database by querying the uuid, returning a
    /// Refresh Token instance or sqlx error.
    ///
    /// # Parameters
    ///
    /// * `id` - The uuid of the Refresh Token to be returned.
    /// * `database` - The sqlx database pool for the database to be queried.
    /// ---
    #[tracing::instrument(
        name = "Get a Refresh Token from the database using its id (uuid): ",
        skip(id, database),
        fields(
            id = % id,
        )
    )]
    pub async fn from_id(
        id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<RefreshTokens, BackendError> {
        let database_record = sqlx::query_as!(
            RefreshTokens,
            r#"
                    SELECT *
                    FROM refresh_tokens
                    WHERE id = $1
                "#,
            id
        )
        .fetch_one(database)
        .await?;

        tracing::debug!(
            "Refresh Token database records retrieved: {database_record:#?}"
        );

        Ok(database_record)
    }

    /// Get a Refresh Token from the database by querying the uuid, returning a
    /// Refresh Token instance or sqlx error.
    ///
    /// # Parameters
    ///
    /// * `refresh_tokens` - The &str of the Refresh Token in the database.
    /// * `database` - The sqlx database pool for the database to be queried.
    /// ---
    #[tracing::instrument(
        name = "Get a Refresh Token from the database using the token: ",
        skip(refresh_token, database),
    // fields(
    // 	refresh_tokens = %,
    // ),
    )]
    pub async fn from_token(
        refresh_token: &str,
        database: &Pool<Postgres>,
    ) -> Result<RefreshTokens, BackendError> {
        let database_record = sqlx::query_as!(
            RefreshTokens,
            r#"
                    SELECT *
                    FROM refresh_tokens
                    WHERE token = $1
                "#,
            refresh_token
        )
        .fetch_one(database)
        .await?;

        tracing::debug!(
            "Refresh Token database records retrieved: {database_record:#?}"
        );

        Ok(database_record)
    }

    /// Get an index of Refresh Token from the database by querying a User ID (uuid),
    /// returning a Vec of Refresh Tokens or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `user_id` - The uuid of user to be returned.
    /// * `limit` - A i64 limiting the page length
    /// * `offset` - A i64 of where the limit should start
    /// * `database` - The sqlx database pool for the database to be queried.
    /// ---
    #[tracing::instrument(
        name = "Get all Refresh Token from the database for a users id (uuid): ",
        skip(database),
    // fields(
    // 	user_id = %user_id,
    // )
    )]
    pub async fn index_from_user_id(
        user_id: &Uuid,
        limit: &i64,
        offset: &i64,
        database: &Pool<Postgres>,
    ) -> Result<Vec<RefreshTokens>, BackendError> {
        let database_records = sqlx::query_as!(
            RefreshTokens,
            r#"
                    SELECT *
                    FROM refresh_tokens
                    WHERE user_id = $1
                    ORDER BY id
                    LIMIT $2 OFFSET $3
                "#,
            user_id,
            limit,
            offset,
        )
        .fetch_all(database)
        .await?;

        tracing::debug!(
            "Refresh Token database records retrieved: {database_records:#?}"
        );

        Ok(database_records)
    }

    /// Get an index of Refresh Tokens, returning a vector of Refresh Tokens or
    /// and SQLx error.
    ///
    /// # Parameters
    ///
    /// * `limit` - A i64 limiting the page length
    /// * `offset` - A i64 of where the limit should start
    /// * `database` - The sqlx database pool for the database to be queried.
    /// ---
    #[tracing::instrument(
        name = "Index of Refresh Tokens with offset and limit: ",
        skip(database)
    )]
    pub async fn index(
        limit: &i64,
        offset: &i64,
        database: &Pool<Postgres>,
    ) -> Result<Vec<RefreshTokens>, BackendError> {
        let database_records = sqlx::query_as!(
            RefreshTokens,
            r#"
                    SELECT *
                    FROM refresh_tokens
                    ORDER BY id
                    LIMIT $1 OFFSET $2
                "#,
            limit,
            offset,
        )
        .fetch_all(database)
        .await?;

        tracing::debug!(
            "Refresh Token database records retrieved: {database_records:#?}"
        );

        Ok(database_records)
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

    // Test getting Refresh Token from database using unique UUID
    #[sqlx::test]
    async fn get_refresh_token_record_by_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        // Generate refresh token
        let refresh_token =
            database::RefreshTokens::mock_data(&random_user).await?;

        // Insert refresh token into database for reading later
        refresh_token.insert(&database).await?;

        //-- Execute Function (Act)
        // Insert user into database
        let database_record =
            database::RefreshTokens::from_id(&refresh_token.id, &database).await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        assert_eq!(database_record, refresh_token);

        // -- Return
        Ok(())
    }

    // Test getting Refresh Token from database using unique UUID
    #[sqlx::test]
    async fn get_refresh_token_record_by_token(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        // Generate refresh token
        let refresh_token =
            database::RefreshTokens::mock_data(&random_user).await?;

        // Insert refresh token into database for reading later
        refresh_token.insert(&database).await?;

        //-- Execute Function (Act)
        // Insert user into database
        let database_record = database::RefreshTokens::from_token(
            &refresh_token.token.as_ref(),
            &database,
        )
        .await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        assert_eq!(database_record, refresh_token);

        // -- Return
        Ok(())
    }

    // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn count_index_from_user_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate refresh token
            let refresh_token =
                database::RefreshTokens::mock_data(&random_user).await?;

            // Insert refresh token in the database for deleting
            refresh_token.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Get a random limit from the count
        let random_limit = (1..random_count).fake::<i64>();

        // Get a random offset from the count
        let random_offset = (1..random_count).fake::<i64>();

        // Insert user into database
        let database_records = database::RefreshTokens::index_from_user_id(
            &random_user.id,
            &random_limit,
            &random_offset,
            &database,
        )
        .await?;
        // println!("{rows_affected:#?}");

        //-- Checks (Assertions)
        let count_less_offset: i64 = random_count - random_offset;
        let expected_records = if count_less_offset < random_limit {
            count_less_offset
        } else {
            random_limit
        };

        assert_eq!(database_records.len() as i64, expected_records);

        // -- Return
        Ok(())
    }

    // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn count_index(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate refresh token
            let refresh_token =
                database::RefreshTokens::mock_data(&random_user).await?;

            // Insert refresh token in the database for deleting
            refresh_token.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Get a random limit from the count
        let random_limit = (1..random_count).fake::<i64>();

        // Get a random offset from the count
        let random_offset = (1..random_count).fake::<i64>();

        // Insert user into database
        let database_records =
            database::RefreshTokens::index(&random_limit, &random_offset, &database)
                .await?;
        // println!("{rows_affected:#?}");

        //-- Checks (Assertions)
        let count_less_offset: i64 = random_count - random_offset;
        let expected_records = if count_less_offset < random_limit {
            count_less_offset
        } else {
            random_limit
        };
        assert_eq!(database_records.len() as i64, expected_records);

        // -- Return
        Ok(())
    }
}
