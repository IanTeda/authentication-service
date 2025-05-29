//-- ./src/database/sessions/insert.rs

// #![allow(unused)] // For development only

//! Session insertion logic for the authentication service.
//!
//! This module provides functions to insert session records into the database,
//! including inserting new sessions and returning the created session instance.
//!
//! # Contents
//! - Insert a single session into the database
//! - Unit tests for session insertion logic

use crate::prelude::*;
use sqlx::{Pool, Postgres};

use crate::database;

impl database::Sessions {
    /// Insert a Session into the database, returning sessions instance from the
    /// database.
    ///
    /// # Parameters
    ///
    /// * `self` - A sessions instance
    /// * `database` - An Sqlx database connection pool
    /// ---
    #[tracing::instrument(
        name = "Insert a new Sessions into the database: ",
        skip(database),
        fields(
            session = ?self
        )
    )]
    pub async fn insert(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<Self, AuthenticationError> {
        let database_record = sqlx::query_as!(
            database::Sessions,
            r#"
				INSERT INTO sessions (id, user_id, logged_in_at, login_ip, expires_on, refresh_token, is_active, logged_out_at, logout_ip)
				VALUES ($1, $2, $3, $4, $5, $6, $7,$8, $9) 
				RETURNING *
			"#,
            self.id,
            self.user_id,
            self.logged_in_at,
            self.login_ip,
            self.expires_on,
            self.refresh_token.as_ref(),
            self.is_active,
            self.logged_out_at,
            self.logout_ip
        )
        .fetch_one(database)
        .await?;

        tracing::debug!("Sessions database insert record: {database_record:#?}");

        Ok(database_record)
    }

    #[cfg(test)]
    /// Insert multiple sessions for a given user into the database for testing purposes.
    ///
    /// # Parameters
    /// * `count` - The number of sessions to insert.
    /// * `user` - Reference to the user for whom sessions will be created.
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok((Users, Vec<Sessions>))` - The inserted user and a vector of the inserted session records.
    /// * `Err(AuthenticationError)` - If any insertion fails.
    ///
    /// # Notes
    /// - The user will be inserted into the database if not already present.
    /// - Each session is created using mock data and associated with the provided user.
    pub async fn insert_n_sessions(
        n: usize,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<(database::Users, Vec<database::Sessions>), AuthenticationError>
    {
        // Generate a mock user and insert it into the database
        let user = database::Users::mock_data()?;
        let _db_user = user.insert(database).await?;

        // Generate a mock session and insert n sessions.
        let mut sessions = Vec::new();
        for _ in 0..n {
            let session = database::Sessions::mock_data(&user).await?;
            let db_session = session.insert(database).await?;
            sessions.push(db_session);
        }
        Ok((user, sessions))
    }
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {
    use sqlx::{Pool, Postgres};

    use crate::database;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    // Test inserting into database
    #[sqlx::test]
    async fn create_database_record(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        // Generate session
        let random_session = database::Sessions::mock_data(&random_user).await?;

        //-- Execute Function (Act)
        // Insert session into database
        let database_record = random_session.insert(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(database_record, random_session);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn insert_duplicate_session_id_fails(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Insert a user and one session using the helper
        let (_user, mut sessions) =
            database::Sessions::insert_n_sessions(1, &database).await?;
        let original_session = sessions.pop().expect("Should have one session");

        //-- Execute Function (Act)
        // Attempt to insert another session with the same ID
        let duplicate_session = database::Sessions {
            ..original_session.clone()
        };
        let result = duplicate_session.insert(&database).await;

        //-- Checks (Assertions)
        assert!(
            result.is_err(),
            "Inserting duplicate session ID should fail"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn insert_multiple_sessions_for_same_user(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = database::Users::mock_data()?;
        random_user.insert(&database).await?;
        let session1 = database::Sessions::mock_data(&random_user).await?;
        let session2 = database::Sessions::mock_data(&random_user).await?;

        //-- Execute Function (Act)
        let db_session1 = session1.insert(&database).await?;
        let db_session2 = session2.insert(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(db_session1.user_id, db_session2.user_id);
        assert_ne!(db_session1.id, db_session2.id);

        Ok(())
    }

    #[sqlx::test]
    async fn insert_session_with_null_logout_fields(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = database::Users::mock_data()?;
        random_user.insert(&database).await?;
        let mut session = database::Sessions::mock_data(&random_user).await?;
        session.logged_out_at = None;
        session.logout_ip = None;

        //-- Checks (Assertions)
        let db_session = session.insert(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(db_session.logged_out_at, None);
        assert_eq!(db_session.logout_ip, None);

        Ok(())
    }

    #[sqlx::test]
    async fn insert_session_with_inactive_status(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = database::Users::mock_data()?;
        random_user.insert(&database).await?;
        let mut session = database::Sessions::mock_data(&random_user).await?;

        //-- Checks (Assertions)
        session.is_active = false;
        let db_session = session.insert(&database).await?;

        //-- Checks (Assertions)
        assert!(!db_session.is_active);
        
        Ok(())
    }
}
