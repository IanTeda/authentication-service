//-- ./src/database/users/insert.rs

// #![allow(unused)] // For development only

//! Insert User instance into the database, returning a Result with a User Model instance
//!
//! #### References
//!
//! * [UPDATE query_as! with a custom ENUM type](https://github.com/launchbadge/sqlx/discussions/3041)
//! ---

use crate::database::Users;
use crate::{domain, prelude::*};

impl Users {
    /// Insert a `User` into the database, returning a result with the User database instance created.
    ///
    /// # Parameters
    ///
    /// * `self` - The User instance to be inserted in the database.
    /// * `database` - An Sqlx database connection pool
    /// ---
    #[tracing::instrument(
        name = "Insert a new User into the database: ",
        skip(self, database),
        fields(
            id = % self.id,
            email = % self.email,
            name = % self.name.as_ref(),
            role = % self.role,
            password_hash = % self.password_hash.as_ref(),
            is_active = % self.is_active,
            is_verified = % self.is_verified,
            created_on = % self.created_on,
        ),
    )]
    pub async fn insert(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, AuthenticationError> {
        // Insert a new user
        let database_record = sqlx::query_as!(
            Users,
            r#"
                INSERT INTO users (
                    id,
                    email,
                    name,
                    password_hash,
                    role,
                    is_active,
                    is_verified,
                    created_on
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id, email, name, password_hash, role as "role:domain::UserRole", is_active, is_verified, created_on
            "#,
            self.id,
            self.email.as_ref(),
            self.name.as_ref(),
            self.password_hash.as_ref(),
            self.role.clone() as domain::UserRole,
            self.is_active,
            self.is_verified,
			self.created_on,
        )
            .fetch_one(database)
            .await?;

        tracing::debug!("User database records retrieved: {database_record:#?}");

        Ok(database_record)
    }

    #[cfg(test)]
    // Helper function to insert multiple users into the database
    /// Insert `n` users into the database, returning a vector of the inserted Users.
    pub async fn insert_n_users(
        n: i64,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<Users>, AuthenticationError> {
        let mut users = Vec::new();
        for _ in 0..n {
            let user = Users::mock_data()?;
            let db_user = user.insert(database).await?;
            users.push(db_user);
        }
        Ok(users)
    }
}

//-- Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use sqlx::{Pool, Postgres};

    // Override with more flexible result and error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    // Test inserting into database
    #[sqlx::test]
    async fn create_database_record(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = Users::mock_data()?;

        //-- Execute Function (Act)
        let database_record = random_user.insert(&database).await?;

        //-- Checks (Assertions)
        // Check the two user instances are equal
        assert_eq!(random_user, database_record);

        Ok(())
    }

    #[sqlx::test]
    async fn insert_duplicate_email_fails(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Create two users with the same email. The first user will be inserted
        // successfully. The second user will fail due to duplicate email.
        let user1 = Users::mock_data()?;
        let mut user2 = Users::mock_data()?;
        user2.email = user1.email.clone();
        user1.insert(&database).await?;

        //-- Execute Function (Act)
        // Attempt to insert the second user with the same email. This should fail
        // with a unique constraint violation
        let result = user2.insert(&database).await;

        //-- Checks (Assertions)
        // Assert that the result is an error
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn insert_with_each_role(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Test inserting users with each role
        for role in &[
            domain::UserRole::Admin,
            domain::UserRole::User,
            domain::UserRole::Guest,
        ] {
            let mut user = Users::mock_data()?;
            user.role = role.clone();

            //-- Execute Function (Act)
            // Insert the user with the specified role
            let db_user = user.insert(&database).await?;

            //-- Checks (Assertions)
            // Assert that the role in the database matches the role we set
            assert_eq!(db_user.role, *role);
        }

        Ok(())
    }

    #[sqlx::test]
    async fn insert_inactive_and_unverified(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        // Test inserting a user with is_active and is_verified set to false
        let mut user = Users::mock_data()?;
        user.is_active = false;
        user.is_verified = false;

        //-- Execute Function (Act)
        // Insert the user. This should succeed even if the user is inactive and
        // unverified. The database should not enforce these fields to be true as
        // they are not required by the database schema.
        let db_user = user.insert(&database).await?;

        //-- Checks (Assertions)
        // Assert that the user was inserted with is_active and is_verified set to
        // false.
        assert!(!db_user.is_active);
        assert!(!db_user.is_verified);

        Ok(())
    }

    #[sqlx::test]
    async fn insert_preserves_created_on(database: Pool<Postgres>) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let mut user = Users::mock_data()?;
        let custom_time = Utc.with_ymd_and_hms(2020, 1, 1, 12, 0, 0).unwrap();
        user.created_on = custom_time;

        //-- Execute Function (Act)
        // Insert the user with a custom created_on time. This should succeed and
        // the created_on time should be preserved. The database should not modify
        // the created_on time as it is set explicitly in the user instance.
        let db_user = user.insert(&database).await?;

        //-- Checks (Assertions)
        // Assert that the created_on time in the database matches the custom time we set
        assert_eq!(db_user.created_on, custom_time);

        Ok(())
    }
}
