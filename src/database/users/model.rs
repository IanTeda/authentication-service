//-- ./src/database/users/model.rs

//! The Users model
//! ---

#![allow(unused)] // For development only

use crate::{
	domains::{EmailAddress, UserName},
	prelude::*,
	rpc::proto::CreateUserRequest,
};

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

impl TryFrom<CreateUserRequest> for UserModel {
    type Error = BackendError;

    fn try_from(value: CreateUserRequest) -> Result<Self, Self::Error> {
		let id = Uuid::now_v7();
		let email = EmailAddress::parse(value.email)?;
		let user_name = UserName::parse(value.user_name)?;
		let password_hash = value.password;
		let is_active = value.is_active;
		let created_on = Utc::now();

        Ok(Self { id, email, user_name, password_hash, is_active, created_on })
    }
}
//-- Unit Tests
#[cfg(test)]
pub mod tests {

	// Bring module functions into test scope
	use super::*;

	use fake::faker::boolean::en::Boolean;
	use fake::faker::internet::en::{Password, SafeEmail};
	use fake::faker::name::en::Name;
	use fake::faker::{chrono::en::DateTime, chrono::en::DateTimeAfter, lorem::en::*};
	use fake::Fake;

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

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

		// Generate random safe email address
		let random_email: String = SafeEmail().fake();
		let email = EmailAddress::parse(random_email)?;

		// Generate random name
		let random_name: String = Name().fake();
		let user_name = UserName::parse(random_name)?;

		// Generate random password string
		let password_hash: String = Password(14..255).fake();

		// Generate random boolean value
		let is_active: bool = Boolean(4).fake();

		// Generate random DateTime
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

	#[test]
	fn convert_tonic_request_to_user_model() -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate random user to construct Tonic Request
		let random_user = create_random_user()?;
		// Request will consume random_user so lets clone for asserts
		let tonic_user = random_user.clone();
		// Build Tonic request
		let tonic_request: CreateUserRequest = CreateUserRequest {
			email: tonic_user.email.as_ref().to_string(),
			user_name: tonic_user.user_name.as_ref().to_string(),
			password: tonic_user.password_hash,
			is_active: tonic_user.is_active,
		};
		// println!("{tonic_request:#?}");

		//-- Execute Function (Act)
		// Transform CreateUserRequest into UserModel
		let new_user: UserModel = tonic_request.try_into()?;
		// println!("{new_user:#?}");

		//-- Checks (Assertions)
		assert_ne!(random_user.id, new_user.id); // id is dropped so it is not equal
		assert_eq!(random_user.email, new_user.email);
		assert_eq!(random_user.user_name, new_user.user_name);
		assert_eq!(random_user.password_hash, new_user.password_hash);
		assert_eq!(random_user.is_active, new_user.is_active);
		assert_ne!(random_user.created_on, new_user.created_on); // created_on is dropped so it is not equal

		Ok(())
	}
}