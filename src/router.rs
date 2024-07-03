// -- ./src/rpc/mod.rs

//! RPC module containing endpoint configurations
//!
//! `proto` brings the Protobuf generated files into scope
//! `get_router` returns all the rpc endpoints for building the Tonic server.

// #![allow(unused)] // For development only

use crate::prelude::*;
use crate::reflections;
use crate::rpc::ledger::users_server::UsersServer;
use crate::rpc::ledger::utilities_server::UtilitiesServer;
use crate::services::{UsersService, UtilitiesService};

use sqlx::{Pool, Postgres};
use tonic::transport::{server::Router, Server};

pub fn get_router(database: Pool<Postgres>) -> Result<Router, BackendError> {
	// Build ledger rpc server
	let utilities_server = UtilitiesServer::new(UtilitiesService::default());

	let users_server = UsersServer::new(UsersService::new(database));

	let reflections_server = reflections::get_reflection()?;

	// Build RPC server router
	let router = Server::builder()
		.trace_fn(|_| tracing::info_span!("Tonic"))
		.add_service(reflections_server)
		.add_service(users_server)
		.add_service(utilities_server);

	Ok(router)
}
