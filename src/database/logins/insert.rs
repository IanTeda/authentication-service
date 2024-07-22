//-- ./src/database/logins/insert.rs

// #![allow(unused)] // For development only

//! Insert a Login into the database, returning a result with the Login Model
//! ---

use crate::prelude::BackendError;

use super::Logins;

impl Logins {
    /// Insert a Login into the database, returning a result with the Login database instance created.
    ///
    /// # Parameters
    ///
    /// * `self` - The Login instance to be inserted in the database.
    /// * `database` - An Sqlx database connection pool
    /// ---
    #[tracing::instrument(
        name = "Insert a new Login into the database: ",
        skip(self, database),
    // fields(
    //     id = % self.id,
    //     user_id = % self.email,
    //     name = % self.name.as_ref(),
    //     role = % self.role,
    //     password_hash = % self.password_hash.as_ref(),
    //     is_active = % self.is_active,
    //     is_verified = % self.is_verified,
    //     created_on = % self.created_on,
    // ),
    )]
    pub async fn insert(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, BackendError> {
        // Insert a new user
        let database_record = sqlx::query_as!(
            Logins,
            r#"
                INSERT INTO logins (
                    id,
                    user_id,
                    login_on,
                    login_ip
                )
                VALUES ($1, $2, $3, $4)
                RETURNING *
            "#,
            self.id,
            self.user_id,
            self.login_on,
            self.login_ip,
        )
            .fetch_one(database)
            .await?;

        tracing::debug!("Login database records inserted: {database_record:#?}");

        Ok(database_record)
    }
}

//-- Unit Tests
#[cfg(test)]
mod tests {
    use sqlx::{Pool, Postgres};

    use crate::database;

    use super::*;

    // Override with more flexible result and error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    // Test inserting into database
    #[sqlx::test]
    async fn create_database_record(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = database::Users::mock_data()?;
        let _database_record = random_user.insert(&database).await?;

        let random_login = Logins::mock_data(&random_user.id)?;

        //-- Execute Function (Act)
        let database_record = random_login.insert(&database).await?;
        // println!("{database_record:#?}");

        //-- Checks (Assertions)
        // Check the two login instances are equal
        assert_eq!(database_record, random_login);

        Ok(())
    }
}
