//-- ./src/database/refresh_token/model.rs

//! The Refresh Token data model
//!
//! ---

#![allow(unused)] // For development only

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domains::RefreshToken;

#[derive(Debug, serde::Deserialize, sqlx::FromRow, Clone)]
pub struct RefreshTokenModel {
	pub id: Uuid,
	pub user_id: Uuid,
	pub refresh_token: RefreshToken,
	pub is_active: bool,
	pub created_on: DateTime<Utc>,
}

impl RefreshTokenModel {
	pub async fn new(user_id: &Uuid, refresh_token: &str) -> Self {
		let id = Uuid::now_v7();
		let user_id = user_id.to_owned();
		let refresh_token = RefreshToken::from(refresh_token.to_string());
		let is_active = true;
		let created_on = Utc::now();

		Self {
			id,
			user_id,
			refresh_token,
			is_active,
			created_on,
		}
	}

	#[cfg(test)]
	pub async fn create_random(user_id: &Uuid) -> Result<Self, crate::error::BackendError> {
		use fake::faker::boolean::en::Boolean;
		use fake::faker::{chrono::en::DateTime, chrono::en::DateTimeAfter};
		use fake::Fake;
		use rand::distributions::DistString;
		use secrecy::Secret;

		// use crate::prelude::BackendError;

		//-- Generate Uuid V7
		// Generate random DateTime after UNIX time epoch (00:00:00 UTC on 1 January 1970)
		let random_datetime: DateTime<Utc> =
			DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();

		// Convert datetime to a UUID timestamp
		let random_uuid_timestamp: uuid::Timestamp = uuid::Timestamp::from_unix(
			uuid::NoContext,
			random_datetime.timestamp() as u64,
			random_datetime.timestamp_nanos_opt().unwrap() as u32,
		);

		let random_id = Uuid::new_v7(random_uuid_timestamp);

		let user_id = user_id.to_owned();

		//-- Generate random Refresh JWT
		// Generate random secret string
		let random_secret = rand::distributions::Alphanumeric
			.sample_string(&mut rand::thread_rng(), 60);

		let random_secret = Secret::new(random_secret);

		let refresh_token = RefreshToken::new(&random_secret, &user_id).await?;

		// Generate random boolean value
		let random_is_active: bool = Boolean(4).fake();

		// Generate random DateTime
		let random_created_on: DateTime<Utc> = DateTime().fake();

		Ok(Self {
			id: random_id,
			user_id,
			refresh_token,
			is_active: random_is_active,
			created_on: random_created_on,
		})
	}
}
