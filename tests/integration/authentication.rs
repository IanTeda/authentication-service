//-- ./tests/integration/authentication.rs

//! Module for integration testing the authentication endpoints
//!
//! Endpoints include
//!
//! * `authenticate`: Authenticate user email and password
//! * `refresh_authentication`: Refresh the bearer token after it expires
//! * `update_password`: Update my password
//! * `reset_password`: Reset my forgotten password
//! * `logout`: Log me out

#![allow(unused)] // For beginning only.

use fake::{faker::internet::en::SafeEmail, Fake};
use jsonwebtoken::Validation;
use personal_ledger_backend::{
	configuration::{self, Configuration},
	database::users::insert_user,
	domains::Password,
	rpc::ledger::{authentication_client::AuthenticationClient, AuthenticateRequest},
	utilities::jwt::{self, JwtTypes, JWT_ISSUER},
};
use secrecy::Secret;
use sqlx::{Pool, Postgres};
use tonic::Code;
use uuid::Uuid;

use crate::{
	helpers,
	users::{generate_random_password, generate_random_user},
};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn authenticate_returns_token_with_uuid(database: Pool<Postgres>) -> Result<()> {
	//-- Setup and Fixtures (Arrange)
	// Generate a random user
	let mut random_user = generate_random_user()?;

	// Generate a new password so we have the un-hashed string to authenticate
	let random_password = generate_random_password();
	random_user.password_hash = Password::parse(Secret::new(random_password.clone()))?;

	// Insert random user into database for the server to query
	insert_user(&random_user, &database).await?;

	// Spawn Tonic test server
	let tonic_server = helpers::TonicServer::spawn_server(database).await?;

	// Build Tonic user client, with authentication intercept
	let mut authentication_client = AuthenticationClient::with_interceptor(
		tonic_server.client_channel().await?,
		helpers::authentication_intercept,
	);

	//-- Execute Test (Act)
	// Build tonic request
	let request = tonic::Request::new(AuthenticateRequest {
		email: random_user.email.to_string(),
		password: random_password,
	});

	// Send tonic client request to server
	let response = authentication_client
		.authenticate(request)
		.await?
		.into_inner();
	// println!("{response:#?}");

	//-- Checks (Assertions)
	// Parse server config so we can grab the jwt_secret
	let config = Configuration::parse()?;

	// Build the token validation for decoding, which is non for testing
	let mut validation = Validation::default();

	// Decode in to Claim
	let access_claim = jsonwebtoken::decode::<jwt::Claims>(
		&response.access_token,
		&jsonwebtoken::DecodingKey::from_secret(config.application.jwt_secret.as_bytes()),
		&validation,
	)
	.map(|data| data.claims)?;
	let refresh_claim = jsonwebtoken::decode::<jwt::Claims>(
		&response.refresh_token,
		&jsonwebtoken::DecodingKey::from_secret(config.application.jwt_secret.as_bytes()),
		&validation,
	)
	.map(|data| data.claims)?;

	// println!("{access_claim:#?}");

	// Confirm uuids are the same
	assert_eq!(Uuid::parse_str(&access_claim.sub)?, random_user.id);
	assert_eq!(Uuid::parse_str(&refresh_claim.sub)?, random_user.id);
	// Confirm Access token
	assert_eq!(&access_claim.jty, "Access");
	assert_eq!(&refresh_claim.jty, "Refresh");
	// assert!(matches!(&access_claim.jty, JwtTypes::Access));

	Ok(())
}

#[sqlx::test]
async fn incorrect_password_returns_error(database: Pool<Postgres>) -> Result<()> {
	//-- Setup and Fixtures (Arrange)
	// Generate a random user
	let random_user = generate_random_user()?;

	// Generate an incorrect password
	let incorrect_password = generate_random_password();

	// Insert random user into database for the server to query
	insert_user(&random_user, &database).await?;

	// Spawn Tonic test server
	let tonic_server = helpers::TonicServer::spawn_server(database).await?;

	// Build Tonic user client, with authentication intercept
	let mut authentication_client = AuthenticationClient::with_interceptor(
		tonic_server.client_channel().await?,
		helpers::authentication_intercept,
	);

	//-- Execute Test (Act)
	// Build tonic request
	let request = tonic::Request::new(AuthenticateRequest {
		email: random_user.email.to_string(),
		password: incorrect_password,
	});

	// Send tonic client request to server
	let response = authentication_client
		.authenticate(request)
		.await
		.unwrap_err();
	// println!("{response:#?}");

	//-- Checks (Assertions)
	// Confirm Tonic response status code
	assert_eq!(response.code(), Code::Unauthenticated);

	// Confirm Tonic response message
	assert_eq!(response.message(), "Authentication failed!");

	Ok(())
}


#[sqlx::test]
async fn incorrect_email_returns_error(database: Pool<Postgres>) -> Result<()> {
	//-- Setup and Fixtures (Arrange)
	// Generate a random user
	let mut random_user = generate_random_user()?;

	// Generate a new password so we have the un-hashed string to authenticate
	let random_password = generate_random_password();
	random_user.password_hash = Password::parse(Secret::new(random_password.clone()))?;

	// Insert random user into database for the server to query
	insert_user(&random_user, &database).await?;

	// Spawn Tonic test server
	let tonic_server = helpers::TonicServer::spawn_server(database).await?;

	// Build Tonic user client, with authentication intercept
	let mut authentication_client = AuthenticationClient::with_interceptor(
		tonic_server.client_channel().await?,
		helpers::authentication_intercept,
	);

	//-- Execute Test (Act)
	// Generate an incorrect password
	let incorrect_email = SafeEmail().fake();

	// Build tonic request
	let request = tonic::Request::new(AuthenticateRequest {
		email: incorrect_email,
		password: random_password,
	});

	// Send tonic client request to server
	let response = authentication_client
		.authenticate(request)
		.await
		.unwrap_err();
	// println!("{response:#?}");

	//-- Checks (Assertions)
	// Confirm Tonic response status code
	assert_eq!(response.code(), Code::Unauthenticated);

	// Confirm Tonic response message
	assert_eq!(response.message(), "Authentication failed!");

	Ok(())
}