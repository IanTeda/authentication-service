//-- ./src/database/sessions/delete.rs

//! Delete Session in the database, returning a Result with an u64 of the number
//! of rows affected.
//! ---

// #![allow(unused)] // For development only

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::database::Sessions;
use crate::prelude::*;

impl Sessions {
    /// Delete a Sessions from the database, returning a Result with the number
    /// of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `self` - The Sessions instance to be deleted.
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Delete Sessions instance from the database: ",
        skip(database)
    )]
    pub async fn delete(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
                    Delete
                    FROM sessions
                    WHERE id = $1
                "#,
            self.id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions rows database records deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Delete a Sessions from the database by querying the Sessions uuid,
    /// returning a Result with the number of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `id` - Uuid: The Sessions id to be deleted.
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Delete Sessions from the database: ",
        skip(database)
    )]
    pub async fn delete_by_id(
        id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
                    Delete
                    FROM sessions
                    WHERE id = $1
                "#,
            id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions row database deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Delete all Sessions from the database associated with a user_id,
    /// returning a Result with the number of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `user_id` - Uuid: The user_id for the Sessions to be deleted
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Delete all Users Sessions from the database: ",
        skip(database)
    )]
    pub async fn delete_all_user(
        user_id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
                    Delete
                    FROM sessions
                    WHERE user_id = $1
                "#,
            user_id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions database records deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Delete all Sessions from the database, returning a Result with the u64 
    /// number of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Delete all Sessions from the database: ",
        skip(database)
    )]
    pub async fn delete_all(database: &Pool<Postgres>) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
                Delete
                FROM sessions

            "#,
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions rows deleted from the database: {rows_affected:#?}");

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

    #[sqlx::test]
    async fn delete_self(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        let random_user = random_user.insert(&database).await?;

        // Generate sessions
        let session = database::Sessions::mock_data(&random_user).await?;

        // Insert session in the database for deleting
        let _database_record = session.insert(&database).await?;

        //-- Execute Function (Act)
        // Delete self from the database
        let rows_affected = session.delete(&database).await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 1);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn delete_by_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        let random_user = random_user.insert(&database).await?;

        // Generate session
        let session = database::Sessions::mock_data(&random_user).await?;

        // Insert session in the database for deleting
        let session = session.insert(&database).await?;

        //-- Execute Function (Act)
        // Delete database row
        let rows_affected =
            database::Sessions::delete_by_id(&session.id, &database).await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        // Each id is unique to the row, so I should equal 1
        assert_eq!(rows_affected, 1);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn delete_associated_to_user(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        let random_user = random_user.insert(&database).await?;

        // Add a random number of sessions for the given user
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate session
            let sessions = database::Sessions::mock_data(&random_user).await?;

            // Insert session into the database for deleting
            let sessions = sessions.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Delete all database entries for the random user id
        let rows_affected =
            database::Sessions::delete_all_user(&random_user.id, &database)
                .await?;
        // println!("{rows_affected:#?}");

        //-- Checks (Assertions)
        // Rows affected should equal the random number of rows inserted
        assert_eq!(rows_affected, random_count as u64);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn delete_all(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate a random number of session
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate random user for testing
            let random_user = database::Users::mock_data()?;

            // Insert user in the database
            let random_user = random_user.insert(&database).await?;

            // Generate session
            let sessions = database::Sessions::mock_data(&random_user).await?;

            // Insert session in the database for deleting
            let sessions = sessions.insert(&database).await?;
        }

        //-- Execute Function (Act)
        // Delete all rows in the database table
        let rows_affected = database::Sessions::delete_all(&database).await?;

        //-- Checks (Assertions)
        // Rows affected should equal the number of random entries
        assert_eq!(rows_affected, random_count as u64);

        // -- Return
        Ok(())
    }
}
