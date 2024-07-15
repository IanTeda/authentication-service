//-- ./src/database/refresh_tokens/model.rs

//! The Refresh Token data model
//! ---

// #![allow(unused)] // For development only

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{domain, utils};

#[derive(Debug, serde::Deserialize, sqlx::FromRow, Clone, PartialEq)]
pub struct RefreshTokens {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: domain::RefreshToken,
    pub is_active: bool,
    pub created_on: DateTime<Utc>,
}

impl RefreshTokens {
    #[tracing::instrument(
        name = "Create new database Refresh Token Model instance for: ",
        skip(refresh_token),
    // fields(
    // 	user_email = %email.as_ref(),
    // )
    )]
    pub fn new(user_id: &Uuid, refresh_token: &domain::RefreshToken) -> Self {
        let id = Uuid::now_v7();
        let user_id = user_id.to_owned();
        let token = refresh_token.to_owned();
        let is_active = true;
        let created_on = Utc::now();

        Self {
            id,
            user_id,
            token,
            is_active,
            created_on,
        }
    }

    #[cfg(feature = "mocks")]
    #[cfg(test)]
    pub async fn mock_data(
        user_id: &Uuid,
    ) -> Result<Self, crate::error::BackendError> {
        use fake::faker::boolean::en::Boolean;
        use fake::faker::chrono::en::DateTime;
        use fake::Fake;
        use rand::distributions::DistString;
        use secrecy::Secret;
        use chrono::SubsecRound;

        let random_id = utils::mock_uuid();
        let user_id = user_id.to_owned();
        let random_secret = rand::distributions::Alphanumeric
            .sample_string(&mut rand::thread_rng(), 60);
        let random_secret = Secret::new(random_secret);

        let random_token =
            domain::RefreshToken::new(&random_secret, &user_id).await?;

        // Generate random boolean value
        let random_is_active: bool = Boolean(4).fake();

        // Generate random DateTime
        let random_created_on: DateTime<Utc> = DateTime().fake();
        let random_created_on = random_created_on.round_subsecs(0);

        Ok(Self {
            id: random_id,
            user_id,
            token: random_token,
            is_active: random_is_active,
            created_on: random_created_on,
        })
    }
}
