//-- ./src/database/sessions/model.rs

//! The Sessions database model
//! ---

// #![allow(unused)] // For development only

use chrono::{DateTime, Utc};
use secrecy::Secret;
use uuid::Uuid;

use crate::{database, domain, prelude::BackendError};

#[derive(Debug, serde::Deserialize, sqlx::FromRow, Clone, PartialEq)]
pub struct Sessions {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token: domain::RefreshToken,
    pub is_active: bool,
    pub created_on: DateTime<Utc>,
}

impl Sessions {
    #[tracing::instrument(
        name = "Create new Sessions instance for: ",
        skip_all,
    )]
    pub fn new(user: &database::Users, token_secret: &Secret<String>) -> Result<Self, BackendError> {
        let id = Uuid::now_v7();
        let user_id = user.id.to_owned();
        let refresh_token = domain::RefreshToken::new(token_secret, user)?;
        let is_active = true;
        let created_on = Utc::now();

        Ok(Self {
            id,
            user_id,
            refresh_token,
            is_active,
            created_on,
        })
    }

    #[cfg(test)]
    pub async fn mock_data(
        user: &database::Users,
    ) -> Result<Self, BackendError> {
        use fake::faker::boolean::en::Boolean;
        use fake::faker::chrono::en::DateTime;
        use fake::Fake;
        use rand::distributions::DistString;
        use secrecy::Secret;
        use chrono::SubsecRound;

        use crate::utils;

        let random_id = utils::mock_uuid();
        let user_id = user.id.to_owned();
        let random_secret = rand::distributions::Alphanumeric
            .sample_string(&mut rand::thread_rng(), 60);
        let random_secret = Secret::new(random_secret);

        let random_token =
            domain::RefreshToken::new(&random_secret, user)?;

        // Generate random boolean value
        let random_is_active: bool = Boolean(4).fake();

        // Generate random DateTime
        let random_created_on: DateTime<Utc> = DateTime().fake();
        let random_created_on = random_created_on.round_subsecs(0);

        Ok(Self {
            id: random_id,
            user_id,
            refresh_token: random_token,
            is_active: random_is_active,
            created_on: random_created_on,
        })
    }
}
