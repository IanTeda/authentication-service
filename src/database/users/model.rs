//-- ./src/database/users/model.rs

//! The Users model
//! ---

// #![allow(unused)] // For development only

use crate::{
	domains::{EmailAddress, PasswordHash, UserName},
	prelude::*,
	rpc::ledger::{CreateUserRequest, UpdateUserRequest},
};

use chrono::prelude::*;
use secrecy::Secret;
use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Clone, FromRow, Debug)]
pub struct UserModel {
	pub id: Uuid,
	pub email: EmailAddress,
	pub user_name: UserName,
	pub password_hash: PasswordHash,
	pub is_active: bool,
	pub created_on: DateTime<Utc>,
}

impl TryFrom<CreateUserRequest> for UserModel {
	type Error = BackendError;

	fn try_from(value: CreateUserRequest) -> Result<Self, Self::Error> {
		let id = Uuid::now_v7();
		let email = EmailAddress::parse(value.email)?;
		let user_name = UserName::parse(value.user_name)?;
		let password_hash = PasswordHash::parse(Secret::new(value.password))?;
		let is_active = value.is_active;
		let created_on = Utc::now();

		Ok(Self {
			id,
			email,
			user_name,
			password_hash,
			is_active,
			created_on,
		})
	}
}

impl TryFrom<UpdateUserRequest> for UserModel {
	type Error = BackendError;

	fn try_from(value: UpdateUserRequest) -> Result<Self, Self::Error> {
		let id = Uuid::parse_str(value.id.as_str())?;
		let email = EmailAddress::parse(value.email)?;
		let user_name = UserName::parse(value.user_name)?;
		let password_hash = PasswordHash::parse(Secret::new(
			"Place holder as password is not updated here".to_string(),
		))?;
		let is_active = value.is_active;
		let created_on = Utc::now();

		Ok(Self {
			id,
			email,
			user_name,
			password_hash,
			is_active,
			created_on,
		})
	}
}

impl UserModel {
	#[cfg(test)]
	pub async fn mock_data() -> Result<Self, crate::error::BackendError> {
		use fake::faker::boolean::en::Boolean;
		use fake::faker::chrono::en::{DateTime, DateTimeAfter};
		use fake::faker::internet::en::SafeEmail;
		use fake::faker::name::en::Name;
		use fake::Fake;

		//-- Generate a random id (Uuid V7) by first generating a random timestamp
		// Generate random DateTime after UNIX time epoch (00:00:00 UTC on 1 January 1970)
		let random_datetime: DateTime<Utc> =
			DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();

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

		// Generate random password hash
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "aB1%".repeat(random_count);
		let password = Secret::new(password);
		let password_hash = PasswordHash::parse(password)?;

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
}
