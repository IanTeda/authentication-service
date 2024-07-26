//-- ./src/database/read.rs

// #![allow(unused)] // For development only

//! Read Logins from the database
//! ---

use uuid::Uuid;

use crate::error::BackendError;

use super::Logins;

impl Logins {
    /// Get a Login from the database by querying the Login uuid, returning a Login (Self)
    /// instance or sqlx error.
    ///
    /// # Parameters
    ///
    /// * `id` - The unique uuid of the Login to be returned
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Read a Login from the database: ",
        skip(database),
    // fields(
    //     user_id = % id,
    // ),
    )]
    pub async fn from_id(
        id: &Uuid,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, BackendError> {
        let database_record = sqlx::query_as!(
            Logins,
            r#"
                SELECT *
                FROM logins
                WHERE id = $1
            "#,
            id
        )
            .fetch_one(database)
            .await?;

        tracing::debug!("Login database record retrieved: {database_record:#?}");

        Ok(database_record)
    }

    /// Get Login from the database by querying the User Id, returning a vector of Logins or
    /// sqlx error.
    ///
    /// # Parameters
    ///
    /// * `user_id` - The unique user id
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Read an index of Logins from the database with user_id: ",
        skip(database),
    // fields(
    //     user_id = %user_id,
    // ),
    )]
    pub async fn index_user(
        user_id: &Uuid,
        limit: &i64,
        offset: &i64,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<Logins>, BackendError> {
        let database_records = sqlx::query_as!(
            Logins,
            r#"
					SELECT *
					FROM logins
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

        tracing::debug!("Login database records retrieved: {database_records:#?}");

        Ok(database_records)
    }

    /// Get an index of Logins, returning a vector of Logins
    ///
    /// # Parameters
    ///
    /// * `limit` - An i64 limiting the page length
    /// * `offset` - An i64 of where the limit should start
    /// * `database` - An sqlx database pool that the things will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Index of Logins with offset and limit"
        skip(database)
    )]
    pub async fn index(
        limit: &i64,
        offset: &i64,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<Logins>, BackendError> {
        let database_records = sqlx::query_as!(
            Logins,
            r#"
					SELECT *
					FROM logins
					ORDER BY id
					LIMIT $1 OFFSET $2
				"#,
            limit,
            offset,
        )
            .fetch_all(database)
            .await?;

        tracing::debug!("Login database records retrieved: {database_records:#?}");

        Ok(database_records)
    }
}

//-- Unit Tests
#[cfg(test)]
mod tests {
    use fake::Fake;
    use sqlx::{Pool, Postgres};

    use crate::database;

    use super::*;

    // Override with more flexible result and error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    // Test inserting into database
    #[sqlx::test]
    async fn read_logins_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = database::Users::mock_data()?;
        let _database_record = random_user.insert(&database).await?;

        let random_login = Logins::mock_data(&random_user.id)?;
        let _database_record = random_login.insert(&database).await?;

        //-- Execute Function (Act)
        let database_record =
            Logins::from_id(&random_login.id, &database).await?;
        // println!("{database_record:#?}");

        //-- Checks (Assertions)
        // Check the two login instances are equal
        assert_eq!(database_record, random_login);

        Ok(())
    }

    #[sqlx::test]
    async fn read_logins_user_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = database::Users::mock_data()?;
        let _database_record = random_user.insert(&database).await?;

        // Add a random number of loguin for the given user
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            let random_login = Logins::mock_data(&random_user.id)?;
            // Insert login in the database for deleting
            random_login.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Get a random limit from the count
        let random_limit = (1..random_count).fake::<i64>();

        // Get a random offset from the count
        let random_offset = (1..random_count).fake::<i64>();
        let database_records = Logins::index_user(
            &random_user.id,
            &random_limit,
            &random_offset,
            &database,
        )
            .await?;
        // println!("{database_record:#?}");

        //-- Checks (Assertions)
        let count_less_offset: i64 = random_count - random_offset;
        let expected_records = if count_less_offset < random_limit {
            count_less_offset
        } else {
            random_limit
        };

        assert_eq!(database_records.len() as i64, expected_records);

        Ok(())
    }

    #[sqlx::test]
    async fn read_logins_index(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Add a random number of login for the given user
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            let random_user = database::Users::mock_data()?;
            let _database_record = random_user.insert(&database).await?;
            let random_login = Logins::mock_data(&random_user.id)?;
            // Insert login in the database for deleting
            random_login.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Get a random limit from the count
        let random_limit = (1..random_count).fake::<i64>();
        // Get a random offset from the count
        let random_offset = (1..random_count).fake::<i64>();
        let database_records = Logins::index(
            &random_limit,
            &random_offset,
            &database,
        )
            .await?;
        // println!("{database_record:#?}");

        //-- Checks (Assertions)
        let count_less_offset: i64 = random_count - random_offset;
        let expected_records = if count_less_offset < random_limit {
            count_less_offset
        } else {
            random_limit
        };

        assert_eq!(database_records.len() as i64, expected_records);

        Ok(())
    }
}
