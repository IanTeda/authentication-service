// #![allow(unused)] // For beginning only.

use crate::helpers::*;

use personal_ledger_backend::rpc::ledger::{UpdateUserRequest, UserIndexRequest, UserRequest};
use personal_ledger_backend::rpc::ledger::{users_client::UsersClient, CreateUserRequest};

use chrono::prelude::*;
use secrecy::Secret;
use uuid::Uuid;
use sqlx::{Pool, Postgres};
use fake::faker::boolean::en::Boolean;
use fake::faker::{chrono::en::DateTime, chrono::en::DateTimeAfter};
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use personal_ledger_backend::database::users::{insert_user, UserModel};
use personal_ledger_backend::domains::{EmailAddress, Password, UserName};

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

#[sqlx::test]
async fn read_user_returns_user(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let random_user = generate_random_user()?;
	insert_user(&random_user, &database).await?;
    let tonic_server = spawn_test_server(database).await?;
    // Give the test server a few ms to become available
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    //-- Execute Test (Act)
	let read_user_request = UserRequest { 
		id: random_user.id.to_string()
	};
    let mut client = UsersClient::connect(tonic_server.address).await?;
    let request = tonic::Request::new(read_user_request);
    let response = client.read_user(request)
        .await?
        .into_inner();
    // println!("{response:#?}");

    //-- Checks (Assertions)
	assert_eq!(response.email, random_user.email.to_string());
	assert_eq!(response.user_name, random_user.user_name.to_string());
	assert_eq!(response.is_active, random_user.is_active);

    Ok(())
}

#[sqlx::test]
async fn read_index_returns_users(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let random_user = generate_random_user()?;
	insert_user(&random_user, &database).await?;

	// Get a random number between 10 and 30
	let random_count: i64 = (10..30).fake::<i64>();
	// Initiate vector to store users for assertion
	let mut test_vec: Vec<UserModel> = Vec::new();
	// Iterate for the random count generating a user and adding to vector
	for _count in 0..random_count {
		let random = generate_random_user()?;
		// Insert into database and push return to the vector
		test_vec.push(
			insert_user(&random, &database).await?
		);
	}
	// Spawn a Tonic test server
    let tonic_server = spawn_test_server(database).await?;
    // Give the test server a few ms to become available
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    //-- Execute Test (Act)
	// Generate a random limit and offset based on number of user entries
	let random_limit = (1..random_count).fake::<i64>();
	let random_offset = (1..random_count).fake::<i64>();
	let user_index_request = UserIndexRequest {
		limit: random_limit,
		offset: random_offset
	};
	// Connect to users server
    let mut client = UsersClient::connect(tonic_server.address).await?;
    // Build Tonic client request
	let request = tonic::Request::new(user_index_request);
	// Make a Tonic client request
    let response = client.index_users(request)
        .await?
        .into_inner();
    // println!("{response:#?}");
	let index = response.users;

    //-- Checks (Assertions)
	let count_less_offset: i64 = random_count - random_offset;

	let expected_records = if count_less_offset < random_limit {
		count_less_offset + 1
	} else {
		random_limit
	};

	assert_eq!(index.len() as i64, expected_records);

    Ok(())
}

#[sqlx::test]
async fn updated_user_returns_user(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let random_user = generate_random_user()?;
	insert_user(&random_user, &database).await?;
	let updated_user = generate_random_user()?;

    let tonic_server = spawn_test_server(database).await?;
    // Give the test server a few ms to become available
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    //-- Execute Test (Act)
	let update_user_request = UpdateUserRequest {
		id: random_user.id.to_string(),
        email: updated_user.email.to_string(), 
        user_name: updated_user.user_name.to_string(), 
        is_active: updated_user.is_active, 
    };
    let mut client = UsersClient::connect(tonic_server.address).await?;
    let request = tonic::Request::new(update_user_request.clone());
    let response = client.update_user(request)
        .await?
        .into_inner();
    // println!("{response:#?}");

    //-- Checks (Assertions)
	assert_eq!(response.id, random_user.id.to_string());
	assert_eq!(response.email, update_user_request.email.to_string());
	assert_eq!(response.user_name, update_user_request.user_name.to_string());
	assert_eq!(response.is_active, update_user_request.is_active);

    Ok(())
}


#[sqlx::test]
async fn delete_user_returns_boolean(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let random_user = generate_random_user()?;
	insert_user(&random_user, &database).await?;
    let tonic_server = spawn_test_server(database).await?;
    // Give the test server a few ms to become available
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    //-- Execute Test (Act)
	let read_user_request = UserRequest { 
		id: random_user.id.to_string()
	};
    let mut client = UsersClient::connect(tonic_server.address).await?;
    let request = tonic::Request::new(read_user_request);
    let response = client.delete_user(request)
        .await?
        .into_inner();
    // println!("{response:#?}");

    //-- Checks (Assertions)
	assert!(response.is_deleted);

    Ok(())
}