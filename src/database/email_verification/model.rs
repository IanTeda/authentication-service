//-- ./src/database/email_verification/model.rs

#![allow(unused)] // For development only

//! # Email Verification Model
//!
//! This file defines the `EmailVerification` struct representing an email verification 
//! record in the database, along with serialization, deserialization, and database mapping 
//! traits.
//!
//! Provides constructors for creating new email verification instances, as well as helper methods for
//! generating mock email verification data for testing purposes.
//!
//! # Contents
//! - `EmailVerification` struct definition
//! - Constructor for new email verification instances
//! - Mock data generation for tests

use chrono::{DateTime, Utc, SubsecRound};
use uuid::Uuid;
use crate::{database, domain::{self, RowID}, prelude::AuthenticationError};

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow, Clone, PartialEq)]
pub struct EmailVerification {
    pub id: RowID,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

impl EmailVerification {
    /// Creates a new instance of the EmailVerification struct.
    ///
    /// ## Parameters
    /// - `user: &database::Users` - The user that will be used as a foreign key
    /// - `token: &str` - The verification token
    /// - `expires_at: DateTime<Utc>` - The expiration timestamp for the token
    pub fn new(
        user: &database::Users,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: RowID::new(),
            user_id: user.id,
            token: token.to_owned(),
            expires_at,
            used: false,
            created_at: Utc::now().round_subsecs(0),
        }
    }

    #[cfg(test)]
    /// Creates a mock EmailVerification instance for testing.
    pub fn mock_data(user: &database::Users) -> Self {
        use fake::faker::chrono::en::DateTime as FakeDateTime;
        use fake::faker::internet::en::FreeEmail;
        use fake::faker::lorem::en::Word;
        use fake::Fake;

        let id = RowID::mock();
        let token: String = Word().fake();
        let expires_at: DateTime<Utc> = FakeDateTime().fake();
        let expires_at = expires_at + chrono::Duration::days(1);

        Self {
            id,
            user_id: user.id,
            token,
            expires_at,
            used: false,
            created_at: Utc::now().round_subsecs(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Users;

    #[test]
    fn mock_data_produces_valid_email_verification() {
        let user = Users::mock_data().unwrap();
        let ev = EmailVerification::mock_data(&user);

        assert_eq!(ev.user_id, user.id);
        assert!(!ev.token.is_empty());
        assert!(ev.expires_at > ev.created_at);
        assert!(!ev.used);
    }

    #[test]
    fn new_sets_expected_defaults() {
        let user = Users::mock_data().unwrap();
        let token = "testtoken";
        let expires_at = Utc::now() + chrono::Duration::days(1);

        let ev = EmailVerification::new(&user, token, expires_at);

        assert_eq!(ev.user_id, user.id);
        assert_eq!(ev.token, token);
        assert_eq!(ev.used, false);
        assert!(ev.created_at <= Utc::now());
        assert_eq!(ev.expires_at, expires_at);
    }
}