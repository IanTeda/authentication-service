//-- ./tests/integration/users.rs

//! Module for testing the users endpoints
//!
//! Endpoints include
//!
//! * `create_user`: Create a user in the database
//! * `read_user`: Read a user by id in the database
//! * `index_users`: Index of users
//! * `update_user`: Update a user in the database
//! * `delete_user`: Delete a user in the database

// #![allow(unused)] // For beginning only.

use crate::helpers;

use personal_ledger_backend::database::users::{insert_user, UserModel};
use personal_ledger_backend::domains::{EmailAddress, Password, UserName};
use personal_ledger_backend::rpc::ledger::{users_client::UsersClient, CreateUserRequest};
use personal_ledger_backend::rpc::ledger::{UpdateUserRequest, UserIndexRequest, UserRequest};

use chrono::prelude::*;
use fake::faker::boolean::en::Boolean;
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::faker::{chrono::en::DateTime, chrono::en::DateTimeAfter};
use fake::Fake;
use secrecy::Secret;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

pub fn generate_random_password() -> String {
	let random_count = (5..30).fake::<i64>() as usize;
	"aB1%".repeat(random_count)
}

/// Generate a user with random data, returning the UserModel
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
	let random_count = (5..30).fake::<i64>() as usize;
	let password = Secret::new("aB1%".repeat(random_count));
	let password_hash = Password::parse(password)?;

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

/// Create a user in the database and assert the returned user data
#[sqlx::test]
async fn create_user_returns_user(database: Pool<Postgres>) -> Result<()> {
	//-- Setup and Fixtures (Arrange)
	// Generate a random user
	let random_user = generate_random_user()?;

	// Spawn Tonic test server
	let tonic_server = helpers::TonicServer::spawn_server(database).await?;

	// Build Tonic user client, with authentication intercept
	let mut tonic_user_client = UsersClient::with_interceptor(
		tonic_server.client_channel().await?,
		helpers::authentication_intercept,
	);

	//-- Execute Test (Act)
	// Build tonic request
	let request = tonic::Request::new(CreateUserRequest {
		email: random_user.email.to_string(),
		user_name: random_user.user_name.to_string(),
		password: random_user.password_hash.to_string(),
		is_active: random_user.is_active,
	});
	// Send tonic client request to server
	let response = tonic_user_client.create_user(request).await?.into_inner();
	// println!("{response:#?}");

	//-- Checks (Assertions)
	// Check response email equals the generated email
	assert_eq!(response.email, random_user.email.to_string());

	// Check the response user_name equals the generated user_name
	assert_eq!(response.user_name, random_user.user_name.to_string());

	// Check the response is_active equals the generated is_active
	assert_eq!(response.is_active, random_user.is_active);

	Ok(())
}

/// Read a user in the database and assert the returned user data
#[sqlx::test]
async fn read_user_returns_user(database: Pool<Postgres>) -> Result<()> {
	//-- Setup and Fixtures (Arrange)
	// Generate a random user
	let random_user = generate_random_user()?;

	// Insert random user into database for the server to query
	insert_user(&random_user, &database).await?;

	// Spawn Tonic test server
	let tonic_server = helpers::TonicServer::spawn_server(database).await?;

	// Build Tonic user client, with authentication intercept
	let mut tonic_user_client = UsersClient::with_interceptor(
		tonic_server.client_channel().await?,
		helpers::authentication_intercept,
	);

	//-- Execute Test (Act)
	// Build user request
	let user_request = tonic::Request::new(UserRequest {
		id: random_user.id.to_string(),
	});

	// Send read user request
	let response = tonic_user_client
		.read_user(user_request)
		.await?
		.into_inner();
	// println!("{response:#?}");

	//-- Checks (Assertions)
	// Check the response email equals the generated email
	assert_eq!(response.email, random_user.email.to_string());

	// Check the response user_name equals the generated user_name
	assert_eq!(response.user_name, random_user.user_name.to_string());

	// Check the response is_active equals the generated is_active
	assert_eq!(response.is_active, random_user.is_active);

	Ok(())
}

/// Check the read user index returns a collection of users
#[sqlx::test]
async fn read_index_returns_users(database: Pool<Postgres>) -> Result<()> {
	//-- Setup and Fixtures (Arrange)
	// Get a random number between 10 and 30
	let random_count: i64 = (10..30).fake::<i64>();

	// Initiate vector to store users for assertion
	let mut test_vec: Vec<UserModel> = Vec::new();

	// Iterate over the random count generating a user and adding, inserting it
	// into the database and pushing the response to the vector
	for _count in 0..random_count {
		let random = generate_random_user()?;
		// Insert into database and push return to the vector
		test_vec.push(insert_user(&random, &database).await?);
	}
	// Spawn a Tonic test server
	let tonic_server = helpers::TonicServer::spawn_server(database).await?;

	// Build Tonic user client, with authentication intercept
	let mut tonic_user_client = UsersClient::with_interceptor(
		tonic_server.client_channel().await?,
		helpers::authentication_intercept,
	);

	//-- Execute Test (Act)
	// Generate a random limit and offset based on number of user entries
	let random_limit = (1..random_count).fake::<i64>();
	let random_offset = (1..random_count).fake::<i64>();
	// Build Tonic client request
	let request = tonic::Request::new(UserIndexRequest {
		limit: random_limit,
		offset: random_offset,
	});
	// Make a Tonic client request
	let response = tonic_user_client.index_users(request).await?.into_inner();
	// println!("{response:#?}");
	let index = response.users;

	//-- Checks (Assertions)
	let count_less_offset: i64 = random_count - random_offset;

	let expected_records = if count_less_offset < random_limit {
		count_less_offset
	} else {
		random_limit
	};

	// Check the number of returned users equals the limit and offset parameters
	assert_eq!(expected_records, index.len() as i64);

	Ok(())
}

// Test the updated user returns the correct updated data
#[sqlx::test]
async fn updated_user_returns_user(database: Pool<Postgres>) -> Result<()> {
	//-- Setup and Fixtures (Arrange)
	// Generate random user for testing
	let original_user = generate_random_user()?;

	// Insert random user into the database for testing
	insert_user(&original_user, &database).await?;

	// Generate a new random user data to use in update
	let updated_user = generate_random_user()?;

	// Spawn Tonic test server
	let tonic_server = helpers::TonicServer::spawn_server(database).await?;

	// Build Tonic user client, with authentication intercept
	let mut tonic_user_client = UsersClient::with_interceptor(
		tonic_server.client_channel().await?,
		helpers::authentication_intercept,
	);

	//-- Execute Test (Act)
	// Build tonic request
	let request = tonic::Request::new(UpdateUserRequest {
		id: original_user.id.to_string(),
		email: updated_user.email.to_string(),
		user_name: updated_user.user_name.to_string(),
		is_active: updated_user.is_active,
	});
	let response = tonic_user_client.update_user(request).await?.into_inner();
	// println!("{response:#?}");

	//-- Checks (Assertions)
	// Check id is unchanged from original, as update shouldn't make this change
	assert_eq!(response.id, original_user.id.to_string());

	// Check email is updated
	assert_eq!(response.email, updated_user.email.to_string());

	// Check user name is updated
	assert_eq!(response.user_name, updated_user.user_name.to_string());

	// Check is_active is updated
	assert_eq!(response.is_active, updated_user.is_active);

	Ok(())
}

/// Check the delete_user returns a boolean when deleting
#[sqlx::test]
async fn delete_user_returns_boolean(database: Pool<Postgres>) -> Result<()> {
	//-- Setup and Fixtures (Arrange)
	// Generate random user
	let random_user = generate_random_user()?;

	// Insert random user into the database for testing
	insert_user(&random_user, &database).await?;

	// Spawn Tonic test server
	let tonic_server = helpers::TonicServer::spawn_server(database).await?;

	// Build Tonic user client, with authentication intercept
	let mut tonic_user_client = UsersClient::with_interceptor(
		tonic_server.client_channel().await?,
		helpers::authentication_intercept,
	);

	//-- Execute Test (Act)
	// Build tonic user request
	let request = tonic::Request::new(UserRequest {
		id: random_user.id.to_string(),
	});
	// Make request
	let response = tonic_user_client.delete_user(request).await?.into_inner();
	// println!("{response:#?}");

	//-- Checks (Assertions)
	// Check response is true
	assert!(response.is_deleted);

	Ok(())
}
