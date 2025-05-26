//-- ./src/database/users/read.rs

//! Read User[s] in the database, returning a Result with a UserModel instance
//! or a Vec[UserModel]
//! ---

// #![allow(unused)] // For development only

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{database::users::Users, domain, prelude::*};

impl Users {
    /// Retrieve a user from the database by their unique UUID.
    ///
    /// This function queries the `users` table for a user with the specified `id`.
    /// If a user with the given UUID exists, it returns a `Users` model instance;
    /// otherwise, it returns an `AuthenticationError`.
    ///
    /// # Parameters
    /// * `id` - The unique UUID of the user to retrieve.
    /// * `database` - The sqlx database pool to query.
    ///
    /// # Returns
    /// * `Ok(Users)` - The user record if found.
    /// * `Err(AuthenticationError)` - If no user is found or the query fails.
    ///
    /// # Tracing
    /// - Adds the `user_id` field to tracing spans for observability.
    #[tracing::instrument(
        name = "Read a User from the database: ",
        skip(id, database),
        fields(
            user_id = ?id,
        ),
    )]
    pub async fn from_user_id(
        id: &Uuid,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, AuthenticationError> {
        let database_record = sqlx::query_as!(
				Users,
				r#"
					SELECT id, email, name, password_hash, role as "role:domain::UserRole", is_active, is_verified, created_on
					FROM users
					WHERE id = $1
				"#,
				id
			)
            .fetch_one(database)
            .await?;

        tracing::debug!("User database record retrieved: {database_record:#?}");

        Ok(database_record)
    }

    /// Get User from the database by querying the User Email, returning a User (Self) instance or
    /// sqlx error.
    ///
    /// # Parameters
    ///
    /// * `email` - The unique email of the User to select
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Read a User from the database: ",
        skip(email, database),
        fields(
            user_email = ?email.as_ref(),
        ),
    )]
    pub async fn from_user_email(
        email: &domain::EmailAddress,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, AuthenticationError> {
        let database_record = sqlx::query_as!(
				Users,
				r#"
					SELECT id, email, name, password_hash, role as "role:domain::UserRole", is_active, is_verified, created_on
					FROM users
					WHERE email = $1
				"#,
				email.as_ref()
			)
            .fetch_one(database)
            .await?;

        tracing::debug!("User database record retrieved: {database_record:#?}");

        Ok(database_record)
    }

    /// Get an index of Users, returning a vector of Users
    ///
    /// # Parameters
    ///
    /// * `limit` - An i64 limiting the page length
    /// * `offset` - An i64 of where the limit should start
    /// * `database` - An sqlx database pool that the things will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Index of Users with offset and limit"
        skip(database),
        fields(
            limit = ?limit,
            offset = ?offset
        )
    )]
    pub async fn index(
        limit: &i64,
        offset: &i64,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<Users>, AuthenticationError> {
        let database_records = sqlx::query_as!(
				Users,
				r#"
					SELECT id, email, name, password_hash, role as "role:domain::UserRole", is_active, is_verified, created_on
					FROM users
					ORDER BY id
					LIMIT $1 OFFSET $2
				"#,
				limit,
				offset,
			)
            .fetch_all(database)
            .await?;

        tracing::debug!("User database records retrieved: {database_records:#?}");

        Ok(database_records)
    }

    /// Get a page of Users using cursor-based pagination.
    ///
    /// This function returns a vector of Users that come after the provided cursor,
    /// defined by the combination of `created_on` and `id`. This enables stable and
    /// efficient pagination, even as new users are added or removed.
    ///
    /// # Parameters
    /// * `last_created_on` - The `created_on` timestamp of the last user from the previous page (the cursor).
    /// * `last_id` - The `id` of the last user from the previous page (the cursor).
    /// * `limit` - The maximum number of users to return.
    /// * `database` - The sqlx database pool to query.
    ///
    /// # Returns
    /// * `Ok(Vec<Users>)` - A vector of users after the given cursor, ordered by `(created_on, id)`.
    /// * `Err(AuthenticationError)` - If the query fails.
    ///
    /// # Notes
    /// - Uses the `(created_on, id)` composite index for efficient pagination.
    /// - This approach avoids issues with offset-based pagination such as skipping or duplicating records if the dataset changes between queries.
    #[tracing::instrument(
        name = "Index of Users using created on and id cursor position"
        skip(database),
        fields(
            last_created_on = ?last_created_on,
            last_id = ?last_id,
            limit = ?limit,
        )
    )]
    pub async fn index_cursor(
        last_created_on: &DateTime<Utc>,
        last_id: &Uuid,
        limit: &i64,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<Users>, AuthenticationError> {
        let database_records = sqlx::query_as!(
				Users,
                r#"
                SELECT id, email, name, password_hash, role as "role:domain::UserRole", is_active, is_verified, created_on
                FROM users
                WHERE (created_on, id) > ($1, $2)
                ORDER BY created_on, id
                LIMIT $3
            "#,
            last_created_on,
            last_id,
            limit
			)
            .fetch_all(database)
            .await?;

        tracing::debug!("User database records retrieved: {database_records:#?}");

        Ok(database_records)
    }
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{database, domain::EmailAddress};
    use fake::Fake;
    use sqlx::{Pool, Postgres};

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    // Test getting user from database using unique UUID
    #[sqlx::test]
    async fn get_user_record_by_id(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        //-- Execute Function (Act)
        // Retrieve User from database
        let database_record =
            database::Users::from_user_id(&random_user.id, &database).await?;

        //-- Checks (Assertions)
        assert_eq!(database_record, random_user);

        // -- Return
        Ok(())
    }

    // Test getting user from database using unique email
    #[sqlx::test]
    async fn get_user_record_by_email(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate random user for testing
        let random_user = database::Users::mock_data()?;

        // Insert user in the database
        random_user.insert(&database).await?;

        //-- Execute Function (Act)
        // Insert user into database
        let database_record =
            database::Users::from_user_email(&random_user.email, &database).await?;
        // println!("{record:#?}");

        //-- Checks (Assertions)
        assert_eq!(database_record, random_user);

        // -- Return
        Ok(())
    }

    #[sqlx::test]
    async fn default_user_migration(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let default_user_email =
            domain::EmailAddress::parse("default_ams@teda.id.au")?;

        //-- Execute Function (Act)
        // Insert user into database
        let database_record =
            database::Users::from_user_email(&default_user_email, &database).await?;

        //-- Checks (Assertions)
        assert_eq!(database_record.email, default_user_email);

        // -- Return
        Ok(())
    }

    // Test thing query
    #[sqlx::test]
    async fn get_users_in_database(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Pick a random number of users to add to the database
        let random_count: i64 = (10..30).fake::<i64>();

        // Insert random number of users into the database
        let _ = database::Users::insert_n_users(random_count, &database).await?;

        //-- Execute Function (Act)
        let random_limit = (1..random_count).fake::<i64>();
        let random_offset = (1..random_count).fake::<i64>();
        let database_records =
            database::Users::index(&random_limit, &random_offset, &database).await?;

        //-- Checks (Assertions)
        // Calculate the count less offset
        // We need to add the default user that the database migration creates
        let count_less_offset: i64 = (random_count - random_offset + 1);

        // Expected records based on offset and limit
        let expected_records = if count_less_offset < random_limit {
            count_less_offset
        } else {
            random_limit
        };

        // Assert expected length is the same
        assert_eq!(database_records.len() as i64, expected_records);

        // TODO: Fix this test
        // Database records are not in the same order
        // let random_vec_index = (1..expected_records).fake::<i64>() as usize;
        // assert_eq!(database_records[random_vec_index], test_vec[random_vec_index]);

        Ok(())
    }

    #[sqlx::test]
    async fn index_cursor_returns_correct_users(
        database: Pool<Postgres>,
    ) -> Result<()> {
        // Database migration creates a default user, so clear the users table to ensure
        // there are no users in the database
        sqlx::query!("TRUNCATE TABLE users CASCADE")
            .execute(&database)
            .await?;

        // Insert 5 users with increasing created_on timestamps
        let mut users = database::Users::insert_n_users(5, &database).await?;

        for i in 0..5 {
            let mut user = database::Users::mock_data()?;
            // Ensure created_on is strictly increasing
            user.created_on = Utc::now() + chrono::Duration::seconds(i);
            let db_user = user.insert(&database).await?;
            users.push(db_user);
        }
        // Sort users by (created_on, id) to match the query order
        users.sort_by_key(|u| (u.created_on, u.id));

        // Use the first user's created_on and id as the cursor
        let last_created_on = users[1].created_on;
        let last_id = users[1].id;
        let limit = 2;

        //-- Execute Function (Act)
        // Get users after the cursor
        let result = database::Users::index_cursor(
            &last_created_on,
            &last_id,
            &limit,
            &database,
        )
        .await?;

        // Should return the next two users in order
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], users[2]);
        assert_eq!(result[1], users[3]);

        Ok(())
    }

    #[sqlx::test]
    async fn get_user_by_nonexistent_id_returns_error(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Generate a random user that does not exist in the database to use its id
        let random_user = Users::mock_data()?;

        //-- Execute Function (Act)
        // Attempt to retrieve a user by the random id. This should return an error
        // since the user does not exist. The error should be a NotFound error
        let result = database::Users::from_user_id(&random_user.id, &database).await;

        //-- Checks (Assertions)
        // Assert that the result is an error
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn get_user_by_nonexistent_email_returns_error(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = Users::mock_data()?;

        //-- Execute Function (Act)
        // Attempt to retrieve a user by the random email. This should return an error
        // since the user does not exist. The error should be a NotFound error.
        let result =
            database::Users::from_user_email(&random_user.email, &database).await;

        //-- Checks (Assertions)
        // Assert that the result is an error
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn index_returns_empty_when_no_users(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Database migration creates a default user, so clear the users table to ensure
        // there are no users in the database
        sqlx::query!("TRUNCATE TABLE users CASCADE")
            .execute(&database)
            .await?;

        let limit = 10;
        let offset = 0;

        //-- Execute Function (Act)
        // Attempt to retrieve an index of users when there are no users in the database.
        // This should return an empty vector
        let users = database::Users::index(&limit, &offset, &database).await?;

        //-- Checks (Assertions)
        // Assert that the returned vector is empty
        assert!(users.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn index_respects_limit_and_offset(
        database: Pool<Postgres>,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Insert 5 users
        let users = database::Users::insert_n_users(5, &database).await?;
        let limit = 2;
        let offset = 1;

        //-- Execute Function (Act)
        // Retrieve an index of users with a limit and offset. This should return 2
        // users starting from the second user. The users should be ordered by id.
        let users = database::Users::index(&limit, &offset, &database).await?;

        //-- Checks (Assertions)
        // Assert that the returned vector has the correct number of users. We inserted 5
        // users, so with a limit of 2 and an offset of 1, we should get 2 users
        assert_eq!(users.len(), 2);
        // The users should be ordered by id
        let all_users = database::Users::index(&10, &0, &database).await?;
        assert_eq!(users[0], all_users[1]);
        assert_eq!(users[1], all_users[2]);

        Ok(())
    }
}
