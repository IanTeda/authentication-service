//-- ./src/database/users/read.rs

//! Read User[s] int the database, returning a Result with a UserModel instance
//! or a Vec[UserModel]
//! ---

// #![allow(unused)] // For development only

use uuid::Uuid;

use crate::{database::users::Users, domain, prelude::*};

impl Users {
    /// Get a User from the database by querying the User uuid, returning a User Model (Self)
    /// instance or sqlx error.
    ///
    /// # Parameters
    ///
    /// * `id` - The unique uuid of the User to be returned
    /// * `database` - An sqlx database pool that the thing will be searched in.
    /// ---
    #[tracing::instrument(
        name = "Read a User from the database: ",
        skip(id, database),
        fields(
            user_id = % id,
        ),
    )]
    pub async fn from_user_id(
        id: &Uuid,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, BackendError> {
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
            user_email = % email.as_ref(),
        ),
    )]
    pub async fn from_user_email(
        email: &domain::EmailAddress,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, BackendError> {
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
        skip(database)
    )]
    pub async fn index(
        limit: &i64,
        offset: &i64,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<Users>, BackendError> {
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

    // Test getting user from database using unique UUID
    #[cfg(feature = "mocks")]
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
    #[cfg(feature = "mocks")]
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

    // Test thing query
    #[cfg(feature = "mocks")]
    #[sqlx::test]
    async fn get_users_in_database(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Pick a random number of users to add to the database
        let random_count: i64 = (10..30).fake::<i64>();

        // Initiate a vector to store the users
        // let mut test_vec: Vec<database::User> = Vec::new();
        for _count in 0..random_count {
            // Generate random user for testing
            let random_user = database::Users::mock_data()?;

            // Insert user into database
            random_user.insert(&database).await?;

            // Insert user in the database
            // test_vec.push(random_user.insert(&database).await?);
        }

        //-- Execute Function (Act)
        let random_limit = (1..random_count).fake::<i64>();
        let random_offset = (1..random_count).fake::<i64>();
        let database_records =
            database::Users::index(&random_limit, &random_offset, &database).await?;

        //-- Checks (Assertions)
        // Calculate the count less offset
        let count_less_offset: i64 = random_count - random_offset;

        // Expected records based on offset and limit
        let expected_records = if count_less_offset < random_limit {
            count_less_offset
        } else {
            random_limit
        };

        // Assert expected length is the same
        assert_eq!(database_records.len() as i64, expected_records);

        // Database records are not in the same order
        // let random_vec_index = (1..expected_records).fake::<i64>() as usize;
        // assert_eq!(database_records[random_vec_index], test_vec[random_vec_index]);

        Ok(())
    }
}
