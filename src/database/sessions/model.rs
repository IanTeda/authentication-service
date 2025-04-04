//-- ./src/database/sessions/model.rs

//! The Sessions database model
//! ---

// #![allow(unused)] // For development only

use chrono::{DateTime, SubsecRound, Utc};
use std::{net::Ipv4Addr, time};
use secrecy::Secret;
use uuid::Uuid;

use crate::{database, domain, prelude::BackendError};

#[derive(Debug, serde::Deserialize, sqlx::FromRow, Clone, PartialEq)]
pub struct Sessions {
    pub id: Uuid,
    pub user_id: Uuid,
    pub login_on: DateTime<Utc>,
    pub login_ip: Option<i32>,
    pub expires_on: DateTime<Utc>,
    pub refresh_token: domain::RefreshToken,
    pub is_active: bool,
    pub logout_on: Option<DateTime<Utc>>,
    pub logout_ip: Option<i32>,
}

impl Sessions {
    /// # New Database Sessions Instance
    /// 
    /// Creates a new instance of the Sessions struct.
    /// This can be used to insert, update or delete sesions from the database
    /// 
    /// ## Parameters
    /// 
    /// - `user: &database::Users` - The user that will be used as a foreign key
    /// - `login_ip: &Option<i32>` - The IP address of the user request
    /// - `duration: &time::Duration` - The duration of the session. This is the same as the refresh token and cookie duration
    /// - `refresh_token: &domain::RefreshToken` - The refresh token associated with the sessions. Used to request a new access token.
    #[tracing::instrument(name = "Create new Sessions instance for: ", skip_all)]
    pub fn new(
        user: &database::Users,
        login_ip: &Option<i32>,
        duration: &time::Duration,
        refresh_token: &domain::RefreshToken,
    ) -> Result<Self, BackendError> {
        /// The unique (primary key) session id as a UUid v7
        let id = Uuid::now_v7();

        /// The unique user id (foriegn key) for the session user
        let user_id = user.id.to_owned();

        /// The login time is the current time
        let login_on = Utc::now().round_subsecs(0);

        /// The login IP address is the IP address of the user request
        let login_ip = login_ip.to_owned();

        /// The refresh token is the refresh token associated with the sessions
        let refresh_token = refresh_token.to_owned();

        /// The login expires on is the login time + the duration of the session
        let expires_on = login_on + *duration;

        /// The is_active field is set to true by default and can be used revoke a session and thus refresh token.
        let is_active = true;

        /// When did the user logout of the session. Optional as they may not have logged out. So the session expires.
        let logout_on = None;

        /// The  IP address from were the logout request is sent. Optional as they may not have logged out. So the session expires.
        let logout_ip = None;

        Ok(Self {
            id,
            user_id,
            login_on,
            login_ip,
            expires_on,
            refresh_token,
            is_active,
            logout_on,
            logout_ip,
        })
    }

    #[cfg(test)]
    /// # Mock Session Data
    /// 
    /// This function is only available in test mode `#[cfg(test)]`.
    /// Creates a new instance of the Sessions struct with random data.
    /// This can be used to test the database model.
    /// 
    /// ## Parameters
    /// 
    /// `user: &database::Users` - The user that will be used as a foreign key
    pub async fn mock_data(user: &database::Users) -> Result<Self, BackendError> {
        use chrono::SubsecRound;
        use fake::faker::boolean::en::Boolean;
        use fake::faker::chrono::en::DateTime;
        use fake::faker::company::en::CompanyName;
        use fake::faker::internet::en::IPv4;
        use fake::Fake;
        use rand::distributions::DistString;
        use secrecy::Secret;

        use crate::utils;

        // Generate random Uuid V7
        let random_id = utils::mock_uuid();

        // Take ownership of the user_id
        let user_id = user.id.to_owned();

        // Generate a random refresh token
        let random_refresh_token = domain::RefreshToken::mock_data(user)?;

        // Generate random login time
        let random_login_on: DateTime<Utc> = DateTime().fake();
        let random_login_on = random_login_on.round_subsecs(0);

        // Generate random IPV4 address, with 25% chance of being None
        let random_ip: Ipv4Addr = IPv4().fake();
        // Convert IPV4 to an i32 to be consistent with Postgres INT type
        let random_ip = u32::from(random_ip) as i32;
        let random_login_ip = if Boolean(4).fake() {
            Some(random_ip)
        } else {
            None
        };

        // Generate a random expiration date between 1 and 30 days.
        let random_duration_days = (1..30).fake::<u64>();
        let duration = time::Duration::from_secs(random_duration_days * 24 * 60 * 60);
        let random_expires_on = random_login_on + duration;

        // Generate random boolean value
        let random_is_active: bool = Boolean(4).fake();

        // Generate random login time
        let random_logout_on: DateTime<Utc> = DateTime().fake();
        let random_logout = random_login_on.round_subsecs(0);
        let random_logout_on = if Boolean(4).fake() {
            Some(random_logout)
        } else {
            None
        };

        // Generate random IPV4 address, with 25% chance of being None
        let random_ip: Ipv4Addr = IPv4().fake();
        // Convert IPV4 to an i32 to be consistent with Postgres INT type
        let random_ip = u32::from(random_ip) as i32;
        let random_logout_ip = if Boolean(4).fake() {
            Some(random_ip)
        } else {
            None
        };

        // Build the mock session instance
        let mock_session = Self {
            id: random_id,
            user_id,
            login_on: random_login_on,
            login_ip: random_login_ip,
            expires_on: random_expires_on,
            refresh_token: random_refresh_token,
            is_active: random_is_active,
            logout_on: random_logout_on,
            logout_ip: random_logout_ip,
        };

        Ok(mock_session)
    }
}

// No need for unit tests as the mock data does that for us.