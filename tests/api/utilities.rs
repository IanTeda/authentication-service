//-- ./tests/integration/utilities.rs

//! Module for testing the utilities endpoints
//!
//! Endpoints include
//!
//! * `ping`: For checking the backend server is up and running

// #![allow(unused)] // For beginning only.

use authentication_microservice::rpc::proto::{utilities_service_client::UtilitiesServiceClient as UtilitiesClient, Empty};

use sqlx::{Pool, Postgres};

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn ping_returns_pong(database: Pool<Postgres>) -> Result<()> {
	//-- Setup and Fixtures (Arrange)
	// Spawn Tonic test server
	let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

	// Build Tonic user client, with authentication intercept
	let mut tonic_utilities_client = UtilitiesClient::new(
		tonic_server.client_channel().await?
	);

	//-- Execute Test (Act)
	let request_empty = tonic::Request::new(Empty {});
	let response = tonic_utilities_client
		.ping(request_empty)
		.await?
		.into_inner();
	// println!("{response:#?}");

	//-- Checks (Assertions)
	assert_eq!(response.message, "Pong...");

	Ok(())
}
