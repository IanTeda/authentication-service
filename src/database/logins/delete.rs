//-- ./src/database/delete.rs

// #![allow(unused)] // For development only

//! Delete Login from the database, returning a Result with u64 of the number of rows deleted or an
//! Error
//! ---

use uuid::Uuid;

use crate::error::BackendError;

use super::Logins;

impl Logins {
    /// Delete a Login from the database, returning a Result with the number of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `self` - The Login instance to delete from the database
    /// * `database` - The sqlx database pool that the User will be deleted from.
    /// ---
    #[tracing::instrument(
        name = "Delete a Login instance from the database: ",
        skip(self, database),
        fields(
            login_id = % self.id,
        )
    )]
    pub async fn delete(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
					Delete
					FROM logins
					WHERE id = $1
				"#,
            self.id
        )
            .execute(database)
            .await?
            .rows_affected();

        tracing::debug!("Login database records affected: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Delete a Login from the database using a login_id, returning a Result with the number of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `id` - The Login id to delete from the database
    /// * `database` - The sqlx database pool that the User will be deleted from.
    /// ---
    #[tracing::instrument(
        name = "Delete a Login instance from the database: ",
        skip(database)
    )]
    pub async fn delete_by_id(
        id: &Uuid,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
                    Delete
                    FROM logins
                    WHERE id = $1
                "#,
            id
        )
            .execute(database)
            .await?
            .rows_affected();

        tracing::debug!("Login database records deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Delete all Login from the database for a user_id, returning a Result with the number of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `user_id` - The user_id to delete all the logins for
    /// * `database` - The sqlx database pool that the User will be deleted from.
    /// ---
    #[tracing::instrument(
        name = "Delete all Login instance from the database for user_id: ",
        skip(database)
    )]
    pub async fn delete_all_user_id(
        user_id: &Uuid,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
                    Delete
                    FROM logins
                    WHERE user_id = $1
                "#,
            user_id
        )
            .execute(database)
            .await?
            .rows_affected();

        tracing::debug!("Login database records deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Delete all Login from the database, returning a Result with the number of rows deleted or a sqlx error.
    ///
    /// # Parameters
    ///
    /// * `database` - The sqlx database pool that the User will be deleted from.
    /// ---
    #[tracing::instrument(
        name = "Delete all Login instance from the database: ",
        skip(database)
    )]
    pub async fn delete_all(
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query!(
            r#"
                Delete
                FROM logins
            "#,
        )
            .execute(database)
            .await?
            .rows_affected();

        tracing::debug!("Login database records deleted: {rows_affected:#?}");

        Ok(rows_affected)
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

    #[sqlx::test]
    async fn delete_record(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = database::Users::mock_data()?;
        random_user.insert(&database).await?;

        let random_login = Logins::mock_data(&random_user.id)?;
        let random_login = random_login.insert(&database).await?;

        //-- Execute Function (Act)
        let rows_affected = random_login.delete(&database).await?;
        // println!("{database_record:#?}");

        //-- Checks (Assertions)
        // Check the two login instances are equal
        assert_eq!(rows_affected, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_id_database_record(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = database::Users::mock_data()?;
        let _database_record = random_user.insert(&database).await?;

        let random_login = Logins::mock_data(&random_user.id)?;
        let random_login = random_login.insert(&database).await?;

        //-- Execute Function (Act)
        let rows_affected = Logins::delete_by_id(&random_login.id, &database).await?;
        // println!("{database_record:#?}");

        //-- Checks (Assertions)
        // Check the two login instances are equal
        assert_eq!(rows_affected, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_all_off_user(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = database::Users::mock_data()?;
        let _database_record = random_user.insert(&database).await?;

        // Add a random number of refresh tokens for the given user
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            let random_login = Logins::mock_data(&random_user.id)?;
            // Insert refresh token in the database for deleting
            random_login.insert(&database).await?;
        }

        //-- Execute Function (Act)
        let rows_affected =
            Logins::delete_all_user_id(&random_user.id, &database).await?;
        // println!("{database_record:#?}");

        //-- Checks (Assertions)
        // Check the two login instances are equal
        assert_eq!(rows_affected, random_count as u64);

        Ok(())
    }

    //noinspection DuplicatedCode
    #[sqlx::test]
    async fn delete_all(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            let random_user = database::Users::mock_data()?;
            let _database_record = random_user.insert(&database).await?;

            let random_login = Logins::mock_data(&random_user.id)?;
            // Insert refresh token in the database for deleting
            random_login.insert(&database).await?;
        }

        //-- Execute Function (Act)
        let rows_affected = Logins::delete_all(&database).await?;
        // println!("{database_record:#?}");

        //-- Checks (Assertions)
        // Check the two login instances are equal
        assert_eq!(rows_affected, random_count as u64);

        Ok(())
    }
}
