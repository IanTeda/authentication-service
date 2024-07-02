// #![allow(unused)] // For beginning only.

use crate::helpers::*;

use personal_ledger_backend::rpc::proto::{users_client::UsersClient, CreateUserRequest};

use chrono::prelude::*;
use uuid::Uuid;
use sqlx::{Pool, Postgres};
use fake::faker::boolean::en::Boolean;
use fake::faker::{chrono::en::DateTime, chrono::en::DateTimeAfter};
use fake::faker::internet::en::{Password, SafeEmail};
use fake::faker::name::en::Name;
use fake::Fake;
use personal_ledger_backend::database::users::UserModel;
use personal_ledger_backend::domains::{EmailAddress, UserName};

pub fn generate_random_user() -> Result<UserModel> {
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

#[sqlx::test]
async fn create_user_returns_user(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let random_user = generate_random_user()?;
    let create_user_request = CreateUserRequest { 
        email: random_user.email.to_string(), 
        user_name: random_user.user_name.to_string(), 
        password: random_user.password_hash.to_string(), 
        is_active: random_user.is_active, 
    };
    let tonic_server = spawn_test_server(database).await?;
    // Give the test server a few ms to become available
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    //-- Execute Test (Act)
    let mut client = UsersClient::connect(tonic_server.address).await?;
    let request = tonic::Request::new(create_user_request);
    let response = client.create_user(request)
        .await?
        .into_inner();
    // println!("{response:#?}");

    //-- Checks (Assertions)
	assert_eq!(response.email, random_user.email.to_string());
	assert_eq!(response.user_name, random_user.user_name.to_string());
	assert_eq!(response.is_active, random_user.is_active);

    Ok(())
}