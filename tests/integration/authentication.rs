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

// #![allow(unused)] // For beginning only.

use fake::{faker::internet::en::SafeEmail, Fake};
use personal_ledger_backend::{domains, rpc::ledger::{authentication_client::AuthenticationClient, AuthenticateRequest}};
use secrecy::{ExposeSecret, Secret};
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
	// Generate random user for testing
	let mut random_test_user = generate_random_user()?;

	// Override password so we have the original string for verification
	let random_password = generate_random_password();
	let random_password = Secret::new(random_password);
	let random_password_hash = domains::PasswordHash::parse(random_password.clone())?;
	random_test_user.password_hash = random_password_hash;

	// Spawn Tonic test server
	let tonic_server = helpers::TonicServer::spawn_server(database).await?;

	let token_secret = &tonic_server.clone().config.application.token_secret;


	// Build Tonic user client, with authentication intercept
	let mut authentication_client = AuthenticationClient::with_interceptor(
		tonic_server.client_channel().await?,
		helpers::authentication_intercept,
	);

	//-- Execute Test (Act)
	// Build tonic request
	let request = tonic::Request::new(AuthenticateRequest {
		email: random_test_user.email.to_string(),
		password: random_password.expose_secret().to_string(),
	});

	// Send tonic client request to server
	let response = authentication_client
		.authenticate(request)
		.await?
		.into_inner();
	// println!("{response:#?}");

	//-- Checks (Assertions)
	// Parse server config, so we can grab the jwt_secret
	
	let token_secret = Secret::new(token_secret.to_owned());

	let token_claim = domains::TokenClaim::from_token(&response.access_token, &token_secret).await?;

	// Confirm uuids are the same
	assert_eq!(Uuid::parse_str(&token_claim.sub)?, random_test_user.id);
	// Confirm Access token
	assert_eq!(&token_claim.jty, "Access");
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
	random_user.insert(&database).await?;

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
	assert_eq!(response.message(), "Authentication Failed!");

	Ok(())
}

#[sqlx::test]
async fn incorrect_email_returns_error(database: Pool<Postgres>) -> Result<()> {
	//-- Setup and Fixtures (Arrange)
	// Generate a random user
	let mut random_user = generate_random_user()?;

	// Generate a new password so we have the un-hashed string to authenticate
	let random_password = generate_random_password();
	random_user.password_hash = domains::PasswordHash::parse(Secret::new(random_password.clone()))?;

	// Insert random user into database for the server to query
	random_user.insert(&database).await?;

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
	assert_eq!(response.message(), "Authentication Failed!");

	Ok(())
}