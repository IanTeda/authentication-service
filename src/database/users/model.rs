//-- ./src/database/users/model.rs

//! The Users database model
//! ---

// #![allow(unused)] // For development only

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain;

#[derive(Debug, sqlx::FromRow, serde::Deserialize, serde::Serialize, PartialEq)]
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
    pub fn mock_data() -> Result<Self, crate::prelude::BackendError> {
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

    // #[cfg(test)]
    // pub fn mock_data_with_password(password: String) -> Result<Self, crate::prelude::BackendError> {
    //     use chrono::SubsecRound;
    //     use fake::faker::boolean::en::Boolean;
    //     use fake::faker::chrono::en::DateTime;
    //     use fake::Fake;
    //     use secrecy::Secret;

    //     use crate::utils;

    //     let random_id = utils::mock_uuid();
    //     let random_email = domain::EmailAddress::mock_data()?;
    //     let random_name = domain::UserName::mock_data()?;
    //     // let random_password_hash = domain::PasswordHash::mock_data()?;
    //     let password_hash = domain::PasswordHash::parse(Secret::new(password))?;
    //     let random_user_role = domain::UserRole::mock_data();
    //     let random_is_active: bool = Boolean(4).fake();
    //     let random_is_verified: bool = Boolean(4).fake();
    //     let random_created_on: DateTime<Utc> = DateTime().fake();
    //     // Round sub seconds to be consistent with Postgres accuracy
    //     let random_created_on = random_created_on.round_subsecs(0);

    //     Ok(Users {
    //         id: random_id,
    //         email: random_email,
    //         name: random_name,
    //         password_hash,
    //         role: random_user_role,
    //         is_active: random_is_active,
    //         is_verified: random_is_verified,
    //         created_on: random_created_on,
    //     })
    // }
}
