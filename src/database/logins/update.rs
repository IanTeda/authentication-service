//-- ./src/database/update.rs

// #![allow(unused)] // For development only

//! Update Login in the database
//! ---

use uuid::Uuid;

use crate::error::BackendError;

use super::Logins;

impl Logins {
    /// Update a Login in the database, returning result with the database Login instance.
    ///
    /// # Parameters
    ///
    /// * `self` - A Logins instance
    /// * `database` - An Sqlx database connection pool
    /// ---
    #[tracing::instrument(name = "Update a Login in the database: ", skip(database))]
    pub async fn update(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Logins, BackendError> {
        let database_record = sqlx::query_as!(
            Logins,
            r#"
				UPDATE logins
				SET user_id = $2, login_on = $3, login_ip = $4
				WHERE id = $1
				RETURNING *
			"#,
            self.id,
            self.user_id,
            self.login_on,
            self.login_ip,
        )
            .fetch_one(database)
            .await?;

        tracing::debug!("Login database records retrieved: {database_record:#?}");

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
        let mut random_login = random_login.insert(&database).await?;


        //-- Execute Function (Act)
        let random_login_update = Logins::mock_data(&random_user.id)?;
        random_login.login_on = random_login_update.login_on;
        random_login.login_ip = random_login_update.login_ip;
        let database_record = random_login.update(&database).await?;
        // println!("{database_record:#?}");

        //-- Checks (Assertions)
        // Check the two login instances are equal
        assert_eq!(database_record, random_login);

        Ok(())
    }
}