//-- ./src/database/sessions/model.rs

//! The Sessions database model
//! ---

// #![allow(unused)] // For development only

use chrono::{DateTime, Utc};
use std::time;
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
    // TODO: Would it be better to create the new token outside of the model function?
    #[tracing::instrument(name = "Create new Sessions instance for: ", skip_all)]
    pub fn new(
        token_secret: &Secret<String>,
        issuer: &Secret<String>,
        duration: &time::Duration,
        user: &database::Users,
    ) -> Result<Self, BackendError> {
        let id = Uuid::now_v7();
        let user_id = user.id.to_owned();
        let refresh_token = domain::RefreshToken::new(token_secret, issuer, duration, user)?;
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
    pub async fn mock_data(user: &database::Users) -> Result<Self, BackendError> {
        use chrono::SubsecRound;
        use fake::faker::boolean::en::Boolean;
        use fake::faker::chrono::en::DateTime;
        use fake::faker::company::en::CompanyName;
        use fake::Fake;
        use rand::distributions::DistString;
        use secrecy::Secret;

        use crate::utils;

        let random_id = utils::mock_uuid();
        let user_id = user.id.to_owned();
        let random_secret = rand::distributions::Alphanumeric
            .sample_string(&mut rand::thread_rng(), 60);
        let random_secret = Secret::new(random_secret);

        let random_issuer = CompanyName().fake::<String>();
        let random_issuer = Secret::new(random_issuer);

        // Generate a random duration between 1 and 10 hours
        // TODO: This should not be random
        let random_duration =
            std::time::Duration::from_secs((1..36000).fake::<u64>());

        let random_token =
            domain::RefreshToken::new(&random_secret, &random_issuer, &random_duration, user)?;

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
