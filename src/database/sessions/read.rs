//-- ./src/database/sessions/read.rs

//! Read Sessions in the database, returning a Result with a Session instance
//! or a Vec[Sessions]
//! ---

// // #![allow(unused)] // For development only

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::database::Sessions;
use crate::prelude::*;

impl Sessions {
    /// Get a Sessions from the database by querying the uuid, returning a
    /// Sessions instance or sqlx error.
    ///
    /// # Parameters
    ///
    /// * `id` - The uuid of the Sessions to be returned.
    /// * `database` - The sqlx database pool for the database to be queried.
    /// ---
    #[tracing::instrument(
        name = "Get a Sessions from the database: ",
        skip(database)
    )]
    pub async fn from_id(
        id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<Sessions, BackendError> {
        let database_record = sqlx::query_as!(
            Sessions,
            r#"
                    SELECT *
                    FROM sessions
                    WHERE id = $1
                "#,
            id
        )
        .fetch_one(database)
        .await?;

        tracing::debug!(
            "Sessions database records retrieved: {database_record:#?}"
        );

        Ok(database_record)
    }

    /// Get a Sessions from the database by querying the Sessions, returning a
    /// Sessions instance or sqlx error.
    ///
    /// # Parameters
    ///
    /// * `refresh_tokens` - The &str of the Sessions in the database.
    /// * `database` - The sqlx database pool for the database to be queried.
    /// ---
    #[tracing::instrument(
        name = "Get the session associated with: ",
        skip(database)
    )]
    pub async fn from_token(
        refresh_token: &str,
        database: &Pool<Postgres>,
    ) -> Result<Sessions, BackendError> {
        let database_record = sqlx::query_as!(
            Sessions,
            r#"
                    SELECT *
                    FROM sessions
                    WHERE refresh_token = $1
                "#,
            refresh_token
        )
        .fetch_one(database)
        .await?;

        tracing::debug!(
            "Sessions database records retrieved: {database_record:#?}"
        );

        Ok(database_record)
    }

    /// Get an index of Sessions from the database by querying a User ID (uuid),
    /// returning a Vec of Sessions or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `user_id` - The uuid of user to be returned.
    /// * `limit` - A i64 limiting the page length
    /// * `offset` - A i64 of where the limit should start
    /// * `database` - The sqlx database pool for the database to be queried.
    /// ---
    #[tracing::instrument(
        name = "Get all Sessions from the database for a users id (uuid): ",
        skip(database)
    )]
    pub async fn index_from_user_id(
        user_id: &Uuid,
        limit: &i64,
        offset: &i64,
        database: &Pool<Postgres>,
    ) -> Result<Vec<Sessions>, BackendError> {
        let database_records = sqlx::query_as!(
            Sessions,
            r#"
                    SELECT *
                    FROM sessions
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
            "Sessions database records retrieved: {database_records:#?}"
        );

        Ok(database_records)
    }

    /// Get an index of Sessions, returning a vector of Sessions or
    /// and SQLx error.
    ///
    /// # Parameters
    ///
    /// * `limit` - A i64 limiting the page length
    /// * `offset` - A i64 of where the limit should start
    /// * `database` - The sqlx database pool for the database to be queried.
    /// ---
    #[tracing::instrument(
        name = "Index of Sessions with offset and limit: ",
        skip(database)
    )]
    pub async fn index(
        limit: &i64,
        offset: &i64,
        database: &Pool<Postgres>,
    ) -> Result<Vec<Sessions>, BackendError> {
        let database_records = sqlx::query_as!(
            Sessions,
            r#"
                    SELECT *
                    FROM sessions
                    ORDER BY id
                    LIMIT $1 OFFSET $2
                "#,
            limit,
            offset,
        )
        .fetch_all(database)
        .await?;

        tracing::debug!(
            "Sessions database records retrieved: {database_records:#?}"
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

    #[sqlx::test]
    async fn get_instance_by_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        let random_user = random_user.insert(&database).await?;

        // Generate a session
        let session =
            database::Sessions::mock_data(&random_user).await?;

        // Insert session into database for reading later
        let session = session.insert(&database).await?;

        //-- Execute Function (Act)
        // Insert user into database
        let database_record =
            database::Sessions::from_id(&session.id, &database).await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        assert_eq!(database_record, session);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn session_for_refresh_token(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        // Generate a session
        let session =
            database::Sessions::mock_data(&random_user).await?;

        // Insert session into database for reading later
        let session = session.insert(&database).await?;

        //-- Execute Function (Act)
        // Insert user into database
        let database_record = database::Sessions::from_token(
            &session.refresh_token.as_ref(),
            &database,
        )
        .await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        assert_eq!(database_record, session);

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
            // Generate session
            let session =
                database::Sessions::mock_data(&random_user).await?;

            // Insert session into the database for deleting
            let session = session.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Get a random limit from the count
        let random_limit = (1..random_count).fake::<i64>();

        // Get a random offset from the count
        let random_offset = (1..random_count).fake::<i64>();

        // Insert user into database
        let database_records = database::Sessions::index_from_user_id(
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
            // Generate sessions
            let session =
                database::Sessions::mock_data(&random_user).await?;

            // Insert session into the database for deleting
            let session = session.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Get a random limit from the count
        let random_limit = (1..random_count).fake::<i64>();

        // Get a random offset from the count
        let random_offset = (1..random_count).fake::<i64>();

        // Insert user into database
        let database_records =
            database::Sessions::index(&random_limit, &random_offset, &database)
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
