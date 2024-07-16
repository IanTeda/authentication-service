//-- ./src/domains/user_role.rs

// #![allow(unused)] // For beginning only.

//! User role domain
//!
//! Define the user roles within the application.
//! ---

use crate::prelude::*;

/// Allowable user roles
#[derive(
    Clone,
    Debug,
    PartialEq,
    PartialOrd,
    sqlx::Type,
    serde::Deserialize,
    serde::Serialize,
)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

impl UserRole {
    /// Convert UserRole to a string reference
    pub fn to_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
            UserRole::Guest => "guest",
        }
    }

    /// Mock user role, by picking a random role
    #[cfg(test)]
    pub fn mock_data() -> Self {
        let random_role: UserRole = rand::random();
        random_role
    }
}

/// Random pick during mocking
/// let random_role: UserRole = rand::random();
impl rand::distributions::Distribution<UserRole> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> UserRole {
        match rng.gen_range(0..=2) {
            0 => UserRole::Admin,
            1 => UserRole::User,
            _ => UserRole::Guest,
        }
    }
}

impl std::fmt::Display for UserRole {
    /// Convert a UserRole to a String
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::User => write!(f, "user"),
            UserRole::Guest => write!(f, "guest"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = BackendError;

    fn from_str(input: &str) -> Result<UserRole, Self::Err> {
        match input {
            "Admin" => Ok(UserRole::Admin),
            "admin" => Ok(UserRole::Admin),
            "User" => Ok(UserRole::User),
            "user" => Ok(UserRole::User),
            "Guest" => Ok(UserRole::Guest),
            "guest" => Ok(UserRole::Guest),
            _ => Err(BackendError::UserRole),
        }
    }
}

impl AsRef<str> for UserRole {
    fn as_ref(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
            UserRole::Guest => "guest",
        }
    }
}
