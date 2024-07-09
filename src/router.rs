// -- ./src/rpc/mod.rs

//! RPC module containing endpoint configurations
//!
//! `proto` brings the Protobuf generated files into scope
//! `get_router` returns all the rpc endpoints for building the Tonic server.

// #![allow(unused)] // For development only

use sqlx::Pool;
use sqlx::Postgres;
use std::sync::Arc;

use crate::configuration::Configuration;
use crate::middleware;
use crate::prelude::*;
use crate::reflections;
use crate::rpc::ledger::authentication_server::AuthenticationServer;
use crate::rpc::ledger::users_server::UsersServer;
use crate::rpc::ledger::utilities_server::UtilitiesServer;
use crate::services::{AuthenticationService, UsersService, UtilitiesService};

use tonic::transport::{server::Router, Server};

pub fn get_router(
	database: Pool<Postgres>,
	config: Configuration,
) -> Result<Router, BackendError> {
	// Wraps our database pool an Atomic Reference Counted pointer (Arc). Each instance of
	// the backend will get a pointer to the pool instead of getting a raw copy.
	let database = Arc::new(database);

	// Wrap config in an Atomic Reference Counted (ARC) pointer.
	let config = Arc::new(config);

	// Build Utilities server
	let utilities_server =
		UtilitiesServer::new(UtilitiesService::new(Arc::clone(&config)));

	// Build Users server
	let users_server = UsersServer::with_interceptor(
		UsersService::new(Arc::clone(&database), Arc::clone(&config)),
		middleware::authentication::check_authentication,
	);

	// Build Authentication server
	let auth_server = AuthenticationServer::new(AuthenticationService::new(
		Arc::clone(&database),
		Arc::clone(&config),
	));

	// Build reflections server
	let reflections_server = reflections::get_reflection()?;

	// Build RPC server router
	let router = Server::builder()
		.trace_fn(|_| tracing::info_span!("Tonic"))
		.add_service(reflections_server)
		.add_service(auth_server)
		.add_service(users_server)
		.add_service(utilities_server);

	Ok(router)
}
