//-- ./src/database/users/insert.rs

// #![allow(unused)] // For development only

//! Insert User instance into the database, returning a Result with a User Model instance
//!
//! #### References
//!
//! * [UPDATE query_as! with a custom ENUM type](https://github.com/launchbadge/sqlx/discussions/3041)
//! ---

use crate::{domain, prelude::*};
use crate::database::Users;

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
}

//-- Unit Tests
#[cfg(test)]
mod tests {
    use sqlx::{Pool, Postgres};

    use super::*;

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
        // println!("{database_record:#?}");

        //-- Checks (Assertions)
        // Check the two user instances are equal
        assert_eq!(random_user, database_record);

        // -- Return
        Ok(())
    }
}
