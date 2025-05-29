//-- ./src/database/sessions/model.rs

// #![allow(unused)] // For development only

//! The Sessions database model.
//!
//! Defines the `Sessions` struct representing a session record in the database, along with
//! serialization, deserialization, and database mapping traits.
//!
//! Provides constructors for creating new session instances, as well as helper methods for
//! generating mock session data for testing purposes.
//!
//! # Contents
//! - `Sessions` struct definition
//! - Constructor for new session instances
//! - Mock data generation for tests

use chrono::{DateTime, SubsecRound, Utc};
use std::time;
use uuid::Uuid;

use crate::{database, domain, prelude::AuthenticationError};

#[derive(Debug, serde::Deserialize, sqlx::FromRow, Clone, PartialEq)]
pub struct Sessions {
    pub id: Uuid,
    pub user_id: Uuid,
    pub logged_in_at: DateTime<Utc>,
    pub login_ip: Option<i32>,
    pub expires_on: DateTime<Utc>,
    pub refresh_token: domain::RefreshToken,
    pub is_active: bool,
    pub logged_out_at: Option<DateTime<Utc>>,
    pub logout_ip: Option<i32>,
}

impl Sessions {
    /// # New Database Sessions Instance
    /// 
    /// Creates a new instance of the Sessions struct.
    /// This can be used to insert, update or delete sessions from the database
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
    ) -> Result<Self, AuthenticationError> {
        // The unique (primary key) session id as a UUid v7
        let id = Uuid::now_v7();

        // The unique user id (foreign key) for the session user
        let user_id = user.id.to_owned();

        // The login time is the current time
        let logged_in_at = Utc::now().round_subsecs(0);

        // The login IP address is the IP address of the user request
        let login_ip = login_ip.to_owned();

        // The refresh token is the refresh token associated with the sessions
        let refresh_token = refresh_token.to_owned();

        // The login expires on is the login time + the duration of the session
        let expires_on = logged_in_at + *duration;

        // The is_active field is set to true by default and can be used revoke a session and thus refresh token.
        let is_active = true;

        // When did the user logout of the session. Optional as they may not have logged out. So the session expires.
        let logged_out_at = None;

        // The  IP address from were the logout request is sent. Optional as they may not have logged out. So the session expires.
        let logout_ip = None;

        Ok(Self {
            id,
            user_id,
            logged_in_at,
            login_ip,
            expires_on,
            refresh_token,
            is_active,
            logged_out_at,
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
    pub async fn mock_data(user: &database::Users) -> Result<Self, AuthenticationError> {
        use std::net::Ipv4Addr;

        use chrono::SubsecRound;
        use fake::faker::boolean::en::Boolean;
        use fake::faker::chrono::en::DateTime;
        use fake::faker::internet::en::IPv4;
        use fake::Fake;
        // use rand::distributions::DistString;

        use crate::utils;

        // Generate random Uuid V7
        let random_id = utils::mock_uuid();

        // Take ownership of the user_id
        let user_id = user.id.to_owned();

        // Generate a random refresh token
        let random_refresh_token = domain::RefreshToken::mock_data(user)?;

        // Generate random login time
        let random_logged_in_at: DateTime<Utc> = DateTime().fake();
        let random_logged_in_at = random_logged_in_at.round_subsecs(0);

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
        let random_expires_on = random_logged_in_at + duration;

        // Generate random boolean value
        let random_is_active: bool = Boolean(4).fake();

        // Generate random login time
        // let random_logged_out_at: DateTime<Utc> = DateTime().fake();
        let random_logout = random_logged_in_at.round_subsecs(0);
        let random_logged_out_at = if Boolean(4).fake() {
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
            logged_in_at: random_logged_in_at,
            login_ip: random_login_ip,
            expires_on: random_expires_on,
            refresh_token: random_refresh_token,
            is_active: random_is_active,
            logged_out_at: random_logged_out_at,
            logout_ip: random_logout_ip,
        };

        Ok(mock_session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Users;

    #[tokio::test]
    async fn mock_data_produces_valid_session() {
        let user = Users::mock_data().unwrap();
        let session = Sessions::mock_data(&user).await.unwrap();

        // Check that the session fields are set as expected
        assert_eq!(session.user_id, user.id);
        assert!(session.id != Uuid::nil());
        assert!(session.expires_on > session.logged_in_at);
        assert!(session.refresh_token.as_ref().len() > 0);
    }

    #[tokio::test]
    async fn new_session_sets_expected_defaults() {
        let user = Users::mock_data().unwrap();
        let login_ip = Some(123456789);
        let duration = std::time::Duration::from_secs(3600);
        let refresh_token = crate::domain::RefreshToken::mock_data(&user).unwrap();

        let session = Sessions::new(&user, &login_ip, &duration, &refresh_token).unwrap();

        assert_eq!(session.user_id, user.id);
        assert_eq!(session.login_ip, login_ip);
        assert!(session.is_active);
        assert!(session.logged_out_at.is_none());
        assert!(session.logout_ip.is_none());
        assert_eq!(session.refresh_token, refresh_token);
        assert_eq!(session.expires_on, session.logged_in_at + chrono::Duration::from_std(duration).unwrap());
    }

    #[tokio::test]
    async fn mock_data_randomises_fields() {
        let user = Users::mock_data().unwrap();
        let session1 = Sessions::mock_data(&user).await.unwrap();
        let session2 = Sessions::mock_data(&user).await.unwrap();

        // IDs should be unique
        assert_ne!(session1.id, session2.id);

        // Refresh tokens should be unique
        assert_ne!(session1.refresh_token, session2.refresh_token);
    }

    #[tokio::test]
    async fn session_struct_partial_eq_and_clone_work() {
        let user = Users::mock_data().unwrap();
        let session = Sessions::mock_data(&user).await.unwrap();
        let session_clone = session.clone();
        assert_eq!(session, session_clone);
    }
}