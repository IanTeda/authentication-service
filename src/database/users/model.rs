//-- ./src/database/users/model.rs

//! The Users model
//! ---

#![allow(unused)] // For development only

use crate::domains::{EmailAddress, UserName};

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, FromRow, Debug, PartialEq)]
pub struct UserModel {
	pub id: Uuid,
	pub email: EmailAddress,
	pub user_name: UserName,
	pub password_hash: String, // TODO: start with string
	pub is_active: bool,
	pub created_on: DateTime<Utc>,
}

pub struct NewUser {
	pub email: String,
	pub user_name: String,
	pub password_has: String,
	pub is_active: bool
}

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	// Bring module functions into test scope
	use super::*;

	use fake::faker::boolean::en::Boolean;
	use fake::faker::internet::en::{Password, SafeEmail};
	use fake::faker::name::en::Name;
	use fake::faker::{chrono::en::DateTime, chrono::en::DateTimeAfter, lorem::en::*};
	use fake::Fake;

	pub fn create_random_user() -> Result<UserModel> {
		// Generate random DateTime after UNIX time epoch (00:00:00 UTC on 1 January 1970)
		let random_datetime: DateTime<Utc> = DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();
		// Convert datetime to a UUID timestamp
		let random_uuid_timestamp: uuid::Timestamp = uuid::Timestamp::from_unix(
			uuid::NoContext,
			random_datetime.timestamp() as u64,
			random_datetime.timestamp_nanos_opt().unwrap() as u32,
		);
		// Generate Uuid V7
		let id: Uuid = Uuid::new_v7(random_uuid_timestamp);

		let random_email: String = SafeEmail().fake();
		let email = EmailAddress::parse(random_email)?;

		let random_name: String = Name().fake();
		let user_name = UserName::parse(random_name)?;

		let password_hash: String = Password(14..255).fake();

		let is_active: bool = Boolean(4).fake();

		let created_on: DateTime<Utc> = DateTime().fake();

		let random_user = UserModel {
			id,
			email,
			user_name,
			password_hash,
			is_active,
			created_on,
		};

		Ok(random_user)
	}
}
