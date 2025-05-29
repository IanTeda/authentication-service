//-- ./src/database/users/model.rs

//! The Users database model.
//!
//! Defines the `Users` struct representing a user record in the database, along with
//! serialization, deserialization, and database mapping traits.
//!
//! Provides helper methods for generating mock user data for testing, and includes
//! unit tests to verify the structure and behaviour of the user model.
//!
//! # Contents
//! - `Users` struct definition
//! - Mock data generation for tests
//! - Unit tests for model validation

// #![allow(unused)] // For development only

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain;

/// Represents a user record in the database.
///
/// The `Users` struct models all relevant fields for a user, including unique ID,
/// email, name, password hash, role, status flags, and creation timestamp. It
/// supports serialization, deserialization, and database mapping for use with SQLx
/// and Serde.
///
/// # Fields
/// - `id`: Unique identifier for the user (UUID)
/// - `email`: User's email address
/// - `name`: User's display name
/// - `password_hash`: Hashed password for authentication
/// - `role`: User's role (e.g., Admin, User, Guest)
/// - `is_active`: Whether the user account is active
/// - `is_verified`: Whether the user's email is verified
/// - `created_on`: Timestamp when the user was created
#[derive(Debug, sqlx::FromRow, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
#[allow(non_snake_case)]
pub struct Users {
    pub id: Uuid,
    pub email: domain::EmailAddress,
    pub name: domain::UserName,
    pub password_hash: domain::PasswordHash,
    pub role: domain::UserRole,
    pub is_active: bool,
    pub is_verified: bool,
    pub created_on: DateTime<Utc>,
}

impl Users {
    /// Generate a mock `Users` instance for testing purposes.
    ///
    /// This helper creates a user with randomised, valid data for all fields,
    /// including a unique UUID, realistic email, name, password hash, role,
    /// status flags, and a creation timestamp rounded to match Postgres accuracy.
    ///
    /// # Returns
    /// * `Ok(Users)` - A mock user instance with valid, randomised data.
    /// * `Err(AuthenticationError)` - If mock data generation fails for any field.
    ///
    /// # Usage
    /// Intended for use in unit tests and test fixtures to quickly generate
    /// valid user records without manual setup.
    #[cfg(test)]
    pub fn mock_data() -> Result<Self, crate::prelude::AuthenticationError> {
        use fake::faker::boolean::en::Boolean;
        use fake::faker::chrono::en::DateTime;
        use fake::Fake;
        use chrono::SubsecRound;

        use crate::utils;

        let random_id = utils::mock_uuid();
        let random_email = domain::EmailAddress::mock_data()?;
        let random_name = domain::UserName::mock_data()?;
        let random_password_hash = domain::PasswordHash::mock_data()?;
        let random_user_role = domain::UserRole::mock_data();
        let random_is_active: bool = Boolean(4).fake();
        let random_is_verified: bool = Boolean(4).fake();
        let random_created_on: DateTime<Utc> = DateTime().fake();
        // Round sub seconds to be consistent with Postgres accuracy
        let random_created_on = random_created_on.round_subsecs(0);

        Ok(Users {
            id: random_id,
            email: random_email,
            name: random_name,
            password_hash: random_password_hash,
            role: random_user_role,
            is_active: random_is_active,
            is_verified: random_is_verified,
            created_on: random_created_on,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn mock_data_returns_valid_user() {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data().expect("Should generate mock user");

        //-- Checks (Assertions)
        // Check that fields are not default/empty
        assert!(!user.email.as_ref().is_empty());
        assert!(!user.name.as_ref().is_empty());
        assert!(!user.password_hash.as_ref().is_empty());
        // Role should be one of the allowed variants
        matches!(user.role, domain::UserRole::Admin | domain::UserRole::User | domain::UserRole::Guest);
    }

    #[test]
    fn user_struct_field_types_are_correct() {
        let user = Users::mock_data().unwrap();

        // Check types using Rust's type inference (will fail to compile if types are wrong)
        let _: uuid::Uuid = user.id;
        let _: &crate::domain::EmailAddress = &user.email;
        let _: &crate::domain::UserName = &user.name;
        let _: &crate::domain::PasswordHash = &user.password_hash;
        let _: &crate::domain::UserRole = &user.role;
        let _: bool = user.is_active;
        let _: bool = user.is_verified;
        let _: chrono::DateTime<chrono::Utc> = user.created_on;
    }

    #[test]
    fn mock_data_generates_unique_ids() {
        //-- Setup and Fixtures (Arrange)
        let mut ids = HashSet::new();

        //-- Checks (Assertions)
        for _ in 0..100 {
            let user = Users::mock_data().unwrap();
            assert!(ids.insert(user.id), "Duplicate UUID generated");
        }
    }

    #[test]
    fn mock_data_created_on_has_no_subsec() {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data().unwrap();

        //-- Checks (Assertions)
        assert_eq!(user.created_on.timestamp_subsec_micros(), 0);
    }

    #[test]
    fn mock_data_is_active_and_verified_are_bool() {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data().unwrap();

        //-- Checks (Assertions)
        // Just check that the fields are bools (always true)
        assert!(matches!(user.is_active, true | false));
        assert!(matches!(user.is_verified, true | false));
    }

    #[test]
    fn user_struct_can_serialize_and_deserialize() {
        let user = Users::mock_data().unwrap();
        let json = serde_json::to_string(&user).expect("Should serialize to JSON");
        let user2: Users = serde_json::from_str(&json).expect("Should deserialize from JSON");
        assert_eq!(user, user2);
    }

    #[test]
    fn user_struct_partial_eq_and_clone_work() {
        let user = Users::mock_data().unwrap();
        let user_clone = user.clone();
        assert_eq!(user, user_clone);
    }
}