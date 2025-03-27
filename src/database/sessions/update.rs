//-- ./src/database/sessions/update.rs

// #![allow(unused)] // For development only

//! Update Sessions in the database
//! ---

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::database::Sessions;
use crate::prelude::BackendError;

impl Sessions {
    /// Update a self in the database, returning a result with a Sessions instance
    /// or Sqlx error.
    ///
    /// # Parameters
    ///
    /// * `self` - A Sessions instance.
    /// * `database` - An Sqlx database connection pool.
    /// ---
    #[tracing::instrument(
        name = "Update a Session in the database: ",
        skip(database)
    )]
    pub async fn update(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<Sessions, BackendError> {
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

    /// Revoke (make non-active) self in the database, returning a result
    /// with a Sessions instance or and SQLx error
    ///
    /// # Parameters
    ///
    /// * `self` - A Sessions instance.
    /// * `database` - An Sqlx database connection pool.
    /// ---
    #[tracing::instrument(
        name = "Revoke sessions in the database: ",
        skip(database)
    )]
    pub async fn revoke(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query_as!(
            Sessions,
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

        Ok(rows_affected)
    }

    /// Revoke (make non-active) a Session with a give PK (id) in the database,
    /// returning a result with the number of rows affected or and SQLx error
    ///
    /// # Parameters
    ///
    /// * `id` - Uuid: The database row PK (id).
    /// * `database` - An Sqlx database connection pool.
    /// ---
    #[tracing::instrument(
        name = "Revoke Sessions in the database: ",
        skip(database)
    )]
    pub async fn revoke_by_id(
        id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query_as!(
            Sessions,
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

        Ok(rows_affected)
    }

    /// Revoke (make non-active) all Sessions in the database associated to
    /// the self (user_id), returning a result with the number of rows revoked
    /// or an SQLx error
    ///
    /// # Parameters
    ///
    /// * `self` - Sessions instance with the user_id to revoke.
    /// * `database` - An Sqlx database connection pool.
    /// ---
    #[tracing::instrument(
        name = "Revoke all Sessions associated with Self user_id: ",
        skip(database)
    )]
    pub async fn revoke_associated(
        &self,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query_as!(
            Sessions,
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

        Ok(rows_affected)
    }

    /// Revoke (make non-active) all Sessions in the database for a give user_id,
    /// returning a result with the number Sessions revoked or an SQLx error
    ///
    /// # Parameters
    ///
    /// * `user_id` - The user_id for the Sessions to be revoked.
    /// * `database` - An Sqlx database connection pool.
    /// ---
    #[tracing::instrument(
        name = "Revoke all Sessions in the database: ",
        skip(database)
    )]
    pub async fn revoke_user_id(
        user_id: &Uuid,
        database: &Pool<Postgres>,
    ) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query_as!(
            Sessions,
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

        Ok(rows_affected)
    }

    /// Revoke (make non-active) all Sessions in the database, returning a
    /// result with the number of rows revoked or an SQLx error.
    ///
    /// # Parameters
    ///
    /// * `database` - An Sqlx database connection pool.
    /// ---
    #[tracing::instrument(
        name = "Revoke all Sessions in the database: ",
        skip(database)
    )]
    pub async fn revoke_all(database: &Pool<Postgres>) -> Result<u64, BackendError> {
        let rows_affected = sqlx::query_as!(
            Sessions,
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

        Ok(rows_affected)
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
        assert_eq!(rows_affected, random_count as u64 + 1);

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
        assert_eq!(rows_affected, random_count as u64 + 1);

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
        assert_eq!(rows_affected, random_count as u64 + 1);

        // Get record from the database
        let database_record =
            database::Sessions::from_id(&session.id, &database).await?;

        // Check the is_active status is false
        assert_eq!(database_record.is_active, false);

        // -- Return
        Ok(())
    }
}
