//-- ./src/database/refresh_token/model.rs

//! The Refresh Token data model
//! 
//! ---

#![allow(unused)] // For development only

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, sqlx::FromRow, Clone)]
pub struct RefreshTokenModel {
    pub id: Uuid,
	pub user_id: Uuid,
	pub refresh_token: String,
	pub is_active: bool,
	pub created_on: DateTime<Utc>,
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	use crate::utilities;

// Bring module functions into test scope
	use super::*;

	use fake::faker::boolean::en::Boolean;
	use fake::faker::internet::en::SafeEmail;
	use fake::faker::name::en::Name;
	use fake::faker::{chrono::en::DateTime, chrono::en::DateTimeAfter};
	use fake::Fake;
    use rand::distributions::DistString;

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

    pub fn generate_random_uuid() -> Result<Uuid> {
        // Generate random DateTime after UNIX time epoch (00:00:00 UTC on 1 January 1970)
		let random_datetime: DateTime<Utc> = DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();
		// Convert datetime to a UUID timestamp
		let random_uuid_timestamp: uuid::Timestamp = uuid::Timestamp::from_unix(
			uuid::NoContext,
			random_datetime.timestamp() as u64,
			random_datetime.timestamp_nanos_opt().unwrap() as u32,
		);
		// Generate Uuid V7
		let uuid = Uuid::new_v7(random_uuid_timestamp);

        Ok(uuid)
    }

    pub fn generate_random_token() -> Result<String> {
        // Generate random secret string
		let random_secret = rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 60);

		// Initiate JWT Keys
		let jwt_keys = utilities::jwt::JwtKeys::new(random_secret.as_bytes());

		// Generate Uuid V7
		let uuid_subject = generate_random_uuid()?.to_string();

		// Generate random duration
		let random_duration = (10..30000).fake::<u64>();

		// let jwt_type = JwtTypes::Access;
		let jwt_type: utilities::jwt::JwtTypes = rand::random();

		// Initiate new claim
		let claim = utilities::jwt::Claims::new(
			utilities::jwt::JWT_ISSUER.to_owned(),
			uuid_subject,
			random_duration,
			jwt_type,
		);
		// println!("{claim:#?}");

		let jwt_token = claim.to_jwt(&jwt_keys)?;

        Ok(jwt_token)
    }

	pub fn generate_random_refresh_token(user_id: Uuid) -> Result<RefreshTokenModel> {

		// Generate Uuid V7
		let random_id = generate_random_uuid()?;

        // let random_user_id = generate_random_uuid()?;

		let random_refresh_token = generate_random_token()?;

		// Generate random boolean value
		let random_is_active: bool = Boolean(4).fake();

		// Generate random DateTime
		let random_created_on: DateTime<Utc> = DateTime().fake();

		let random_refresh_token = RefreshTokenModel {
			id: random_id,
			user_id,
			refresh_token: random_refresh_token,
			is_active: random_is_active,
			created_on: random_created_on,
		};

		Ok(random_refresh_token)
	}
}