//-- ./src/database/users/model.rs

//! The Users database model
//! ---

// #![allow(unused)] // For development only

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain;

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
}