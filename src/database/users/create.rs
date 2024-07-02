//-- ./src/database/users/create.rs

//! The Users create [insert] into database
//! ---

#![allow(unused)] // For development only

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{domains::{EmailAddress, UserName}, prelude::*};

use super::model::UserModel;
use crate::rpc::proto::CreateUserRequest;

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

#[derive(serde::Serialize, serde::Deserialize, Clone, sqlx::FromRow)]
pub struct CreatedUser {
	pub id: Uuid,
	pub email: String,
	pub user_name: String,
	pub password_hash: String,
	pub is_active: bool,
	pub created_on: DateTime<Utc>,
}

// pub async fn create(
// 	user: UserModel,
// 	database: &sqlx::Pool<sqlx::Postgres>,
// ) -> Result<UserModel, BackendError> {

    
// 	Ok(database_record)
// }

//-- Unit Tests
#[cfg(test)]
pub mod tests {

	use crate::database::users::model::tests::create_random_user;

	// Bring module functions into test scope
	use super::*;

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	#[test]
	fn convert_create_user_request_to_user_model() -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		let random_user = create_random_user()?;
		let tonic_user = random_user.clone();
		let tonic_request: CreateUserRequest = CreateUserRequest {
			email: tonic_user.email.as_ref().to_string(),
			user_name: tonic_user.user_name.as_ref().to_string(),
			password: tonic_user.password_hash,
			is_active: tonic_user.is_active,
		};
		// println!("{tonic_request:#?}");

		//-- Execute Function (Act)
		let new_user: UserModel = tonic_request.try_into()?;
		// println!("{new_user:#?}");

		//-- Checks (Assertions)
		assert_ne!(random_user.id, new_user.id); // id is dropped
		assert_eq!(random_user.email, new_user.email);
		assert_eq!(random_user.user_name, new_user.user_name);
		assert_eq!(random_user.password_hash, new_user.password_hash);
		assert_eq!(random_user.is_active, new_user.is_active);
		assert_ne!(random_user.created_on, new_user.created_on); // created_on is dropped

		Ok(())
	}
}