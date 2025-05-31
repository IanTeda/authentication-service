//-- ./src/database/sessions/update.rs

// #![allow(unused)] // For development only

//! Session update logic for the authentication service.
//!
//! This module provides functions to update and revoke session records in the database,
//! including updating session fields, revoking individual sessions, revoking all sessions
//! for a user, and revoking all sessions globally.
//!
//! # Contents
//! - Update a session by instance
//! - Revoke (deactivate) a session by instance or ID
//! - Revoke all sessions for a user or globally
//! - Unit tests for update and revoke scenarios

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::database::Sessions;
use crate::prelude::AuthenticationError;

impl Sessions {
    /// Update this session in the database.
    ///
    /// Executes a SQL `UPDATE` statement to modify the session record identified by its `id`,
    /// updating the `user_id`, `refresh_token`, and `is_active` fields.
    ///
    /// # Parameters
    /// * `self` - The `Sessions` instance containing the updated data.
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(Sessions)` - The updated session record as returned from the database.
    /// * `Err(AuthenticationError)` - If the database operation fails.
    ///
    /// # Tracing
    /// - Adds a tracing span for observability.
    #[tracing::instrument(
        name = "Update a Session in the database: ",
        skip(database),
        fields(
            session = ?self,
        )
    )]
    pub async fn update(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<Sessions, AuthenticationError> {
        let database_record = sqlx::query_as!(
            Sessions,
            r#"
				UPDATE sessions 
				SET user_id = $2, refresh_token = $3, is_active = $4
				WHERE id = $1 
				RETURNING *
			"#,
            self.id,
            self.user_id,
            self.refresh_token.as_ref(),
            self.is_active,
        )
        .fetch_one(database)
        .await?;

        tracing::debug!("Sessions database records retrieved: {database_record:#?}");

        Ok(database_record)
    }

    /// Revoke (make non-active) this session in the database.
    ///
    /// Executes a SQL `UPDATE` statement to set `is_active = false` for the session record
    /// identified by this session's `id`.
    ///
    /// # Parameters
    /// * `self` - The `Sessions` instance to be revoked.
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of sessions revoked (should be 1 if the session existed, 0 otherwise).
    /// * `Err(AuthenticationError)` - If the database operation fails.
    ///
    /// # Tracing
    /// - Adds the session `id` to the tracing span for observability.
    #[tracing::instrument(
        name = "Revoke sessions in the database: ",
        skip(database),
        fields(
            session_id = ?self.id,
        )
    )]
    pub async fn revoke(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<usize, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                UPDATE sessions
                SET is_active = false
                WHERE id = $1
            "#,
            self.id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions database records updated: {rows_affected:#?}");

        Ok(rows_affected as usize)
    }

    /// Revoke (make non-active) a session in the database by its unique session ID.
    ///
    /// Executes a SQL `UPDATE` statement to set `is_active = false` for the session record
    /// identified by the provided `id`.
    ///
    /// # Parameters
    /// * `id` - The UUID of the session to be revoked.
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of sessions revoked (rows updated, should be 1 if the session existed, 0 otherwise).
    /// * `Err(AuthenticationError)` - If the database operation fails.
    ///
    /// # Tracing
    /// - Adds the session `id` to the tracing span for observability.
    #[tracing::instrument(
        name = "Revoke Sessions in the database: ",
        skip(database),
        fields(
            session_id = ?id,
        )
    )]
    pub async fn revoke_by_id(
        id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<usize, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                UPDATE sessions
                SET is_active = false
                WHERE id = $1
            "#,
            id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions database records revoked: {rows_affected:#?}");

        Ok(rows_affected as usize)
    }

    /// Revoke (make non-active) all sessions in the database associated with the same user ID as this session.
    ///
    /// Executes a SQL `UPDATE` statement to set `is_active = false` for all session records
    /// that have the same `user_id` as the current session instance.
    ///
    /// # Parameters
    /// * `self` - The `Sessions` instance whose `user_id` will be used for revocation.
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of sessions revoked (rows updated).
    /// * `Err(AuthenticationError)` - If the database operation fails.
    ///
    /// # Tracing
    /// - Adds the `user_id` to the tracing span for observability.
    #[tracing::instrument(
        name = "Revoke all Sessions associated with associated user_id: ",
        skip_all,
        fields(
            user_id = ?self.user_id,
        )
    )]
    pub async fn revoke_associated(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<usize, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                UPDATE sessions
                SET is_active = false
                WHERE user_id = $1
            "#,
            self.user_id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions database records revoked: {rows_affected:#?}");

        Ok(rows_affected as usize)
    }

    /// Revoke (make non-active) all sessions in the database for a given user ID.
    ///
    /// Executes a SQL `UPDATE` statement to set `is_active = false` for all session records
    /// associated with the specified `user_id`.
    ///
    /// # Parameters
    /// * `user_id` - The UUID of the user whose sessions should be revoked.
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of sessions revoked (rows updated).
    /// * `Err(AuthenticationError)` - If the database operation fails.
    ///
    /// # Tracing
    /// - Adds the `user_id` to the tracing span for observability.
    #[tracing::instrument(
        name = "Revoke all Sessions in the database: ",
        skip(database),
        fields(
            user_id = ?user_id,
        )
    )]
    pub async fn revoke_user_id(
        user_id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<usize, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                UPDATE sessions
                SET is_active = false
                WHERE user_id = $1
            "#,
            user_id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Sessions database records updated: {rows_affected:#?}");

        Ok(rows_affected as usize)
    }

    /// Revoke (make non-active) all sessions in the database.
    ///
    /// Executes a SQL `UPDATE` statement to set `is_active = false` for all session records.
    ///
    /// # Parameters
    /// * `database` - The SQLx PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of sessions revoked (rows updated).
    /// * `Err(AuthenticationError)` - If the database operation fails.
    ///
    /// # Tracing
    /// - Adds a tracing span for observability.
    #[tracing::instrument(
        name = "Revoke all Sessions in the database: ",
        skip(database)
    )]
    pub async fn revoke_all(
        database: &Pool<Postgres>,
    ) -> Result<usize, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                UPDATE sessions
                SET is_active = false
            "#,
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!(
            "All Sessions revoked in the database, records updated: {rows_affected:#?}"
        );

        Ok(rows_affected as usize)
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

        // Generate session
        let session = database::Sessions::mock_data(&random_user).await?;

        // Insert session in the database for deleting
        let mut session = session.insert(&database).await?;

        // Generate random user for testing
        let random_user_update = database::Users::mock_data()?;

        // Insert user in the database
        let random_user_update = random_user_update.insert(&database).await?;

        // Generate sessions update data
        let sessions_update =
            database::Sessions::mock_data(&random_user_update).await?;

        // Update Sessions data
        session.user_id = sessions_update.user_id;
        session.refresh_token = sessions_update.refresh_token;
        session.is_active = sessions_update.is_active;

        //-- Execute Function (Act)
        // Update session in the database
        let database_record = session.update(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(database_record, session);

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

        // Generate sessions
        let mut session = database::Sessions::mock_data(&random_user).await?;

        // Set Sessions active
        session.is_active = true;

        // Insert session into database
        let session = session.insert(&database).await?;

        //-- Execute Function (Act)
        // Revoke self in the database
        let rows_affected = session.revoke(&database).await?;

        //-- Checks (Assertions)
        assert_eq!(rows_affected, 1);

        // Get database record from the database
        let database_record =
            database::Sessions::from_id(&session.id, &database).await?;

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

        // Generate session
        let mut session = database::Sessions::mock_data(&random_user).await?;

        // Set Session active to true
        session.is_active = true;

        // Insert session into database
        let session = session.insert(&database).await?;

        //-- Execute Function (Act)
        let rows_affected =
            database::Sessions::revoke_by_id(&session.id, &database).await?;

        //-- Checks (Assertions)
        // The one entry should be affected
        assert_eq!(rows_affected, 1);

        // Get record from the database
        let database_record =
            database::Sessions::from_id(&session.id, &database).await?;

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

        let mut session = database::Sessions::mock_data(&random_user).await?;

        // Set session active to true
        session.is_active = true;

        // Insert session into the database for deleting
        session.insert(&database).await?;

        let mut test_vec: Vec<database::Sessions> = Vec::new();
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate session
            let mut loop_session =
                database::Sessions::mock_data(&random_user).await?;

            // Set Session active to true
            loop_session.is_active = true;

            // Insert session in the database for deleting
            let loop_session = loop_session.insert(&database).await?;

            // Add session to a Vec
            test_vec.push(loop_session);
        }

        //-- Execute Function (Act)
        // Generate an updated session
        let rows_affected = session.revoke_associated(&database).await?;

        //-- Checks (Assertions)
        // There is an extra row outside the loop so we can use it for association
        assert_eq!(rows_affected, random_count as usize + 1);

        // Get record from the database
        let database_record =
            database::Sessions::from_id(&session.id, &database).await?;

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

        let mut session = database::Sessions::mock_data(&random_user).await?;

        // Set session active to true
        session.is_active = true;

        // Insert session in the database for deleting
        let session = session.insert(&database).await?;

        let mut test_vec: Vec<database::Sessions> = Vec::new();
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate session
            let mut loop_session =
                database::Sessions::mock_data(&random_user).await?;

            // Set session active to true
            loop_session.is_active = true;

            // Insert session in the database for deleting
            let loop_session = loop_session.insert(&database).await?;

            // Add session to a Vec
            test_vec.push(loop_session);
        }

        //-- Execute Function (Act)
        // Generate an updated session
        let rows_affected =
            database::Sessions::revoke_user_id(&session.user_id, &database).await?;

        //-- Checks (Assertions)
        // There is an extra row outside the loop so we can use it for association
        assert_eq!(rows_affected, random_count as usize + 1);

        // Get record from the database
        let database_record =
            database::Sessions::from_id(&session.id, &database).await?;

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

        let mut session = database::Sessions::mock_data(&random_user).await?;

        // Set session active to true
        session.is_active = true;

        // Insert session in the database for deleting
        let session = session.insert(&database).await?;

        let mut test_vec: Vec<database::Sessions> = Vec::new();
        let random_count: i64 = (10..30).fake::<i64>();
        for _count in 0..random_count {
            // Generate session
            let mut loop_session =
                database::Sessions::mock_data(&random_user).await?;

            // Set session active to true
            loop_session.is_active = true;

            // Insert Session in the database for deleting
            let loop_session = loop_session.insert(&database).await?;

            // Add Session to a Vec
            test_vec.push(loop_session);
        }

        //-- Execute Function (Act)
        // Generate an updated sessions
        let rows_affected = database::Sessions::revoke_all(&database).await?;

        //-- Checks (Assertions)
        // There is an extra row outside the loop so we can use it for association
        assert_eq!(rows_affected, random_count as usize + 1);

        // Get record from the database
        let database_record =
            database::Sessions::from_id(&session.id, &database).await?;

        // Check the is_active status is false
        assert_eq!(database_record.is_active, false);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn update_nonexistent_session_returns_error(
        database: Pool<Postgres>,
    ) -> Result<()> {
        // Arrange: Create a session that is not in the database
        let random_user = database::Users::mock_data()?;
        let session = database::Sessions::mock_data(&random_user).await?;

        // Act: Try to update the session (should error)
        let result = session.update(&database).await;

        // Assert: Should return an error
        assert!(
            result.is_err(),
            "Updating a nonexistent session should fail"
        );
        Ok(())
    }

    #[sqlx::test]
    async fn revoke_nonexistent_session_returns_zero(
        database: Pool<Postgres>,
    ) -> Result<()> {
        // Arrange: Create a session that is not in the database
        let random_user = database::Users::mock_data()?;
        let session = database::Sessions::mock_data(&random_user).await?;

        // Act: Try to revoke the session (should affect 0 rows)
        let rows_affected = session.revoke(&database).await?;

        // Assert: Should return 0
        assert_eq!(rows_affected, 0);
        Ok(())
    }

    #[sqlx::test]
    async fn revoke_by_id_nonexistent_session_returns_zero(
        database: Pool<Postgres>,
    ) -> Result<()> {
        use uuid::Uuid;
        // Act: Try to revoke a session by a random UUID
        let random_id = Uuid::new_v4();
        let rows_affected =
            database::Sessions::revoke_by_id(&random_id, &database).await?;

        // Assert: Should return 0
        assert_eq!(rows_affected, 0);
        Ok(())
    }

    #[sqlx::test]
    async fn revoke_user_id_nonexistent_user_returns_zero(
        database: Pool<Postgres>,
    ) -> Result<()> {
        use uuid::Uuid;
        // Act: Try to revoke sessions for a random user_id
        let random_user_id = Uuid::new_v4();
        let rows_affected =
            database::Sessions::revoke_user_id(&random_user_id, &database).await?;

        // Assert: Should return 0
        assert_eq!(rows_affected, 0);
        Ok(())
    }

    #[sqlx::test]
    async fn revoke_all_when_no_sessions_returns_zero(
        database: Pool<Postgres>,
    ) -> Result<()> {
        // Ensure the sessions table is empty
        sqlx::query!("TRUNCATE TABLE sessions CASCADE")
            .execute(&database)
            .await?;
        let rows_affected = database::Sessions::revoke_all(&database).await?;

        // Assert: Should return 0
        assert_eq!(rows_affected, 0);
        Ok(())
    }

    #[sqlx::test]
    async fn update_session_no_changes(database: Pool<Postgres>) -> Result<()> {
        // Arrange: Insert a user and session
        let random_user = database::Users::mock_data()?;
        let random_user = random_user.insert(&database).await?;
        let session = database::Sessions::mock_data(&random_user).await?;
        let session = session.insert(&database).await?;

        // Act: Call update without changing any fields
        let updated_session = session.update(&database).await?;

        // Assert: The session should be unchanged
        assert_eq!(updated_session, session);
        Ok(())
    }
}
