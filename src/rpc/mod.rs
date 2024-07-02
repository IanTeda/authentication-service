// -- ./src/rpc/mod.rs

//! RPC module containing endpoint configurations
//! 
//! `proto` brings the Protobuf generated files into scope
//! `get_router` returns all the rpc endpoints for building the Tonic server.

#![allow(unused)] // For development only

pub mod utilities;
pub mod reflections;
pub mod users;

use crate::prelude::*;

use proto::utilities_server::UtilitiesServer;
use proto::users_server::UsersServer;
use sqlx::{Pool, Postgres};
use tonic::transport::{server::Router, Server};
use utilities::UtilitiesService;

// Bring Protobuf generated files into scope
pub mod proto {
	tonic::include_proto!("ledger");
	pub(crate) const DESCRIPTOR_SET: &[u8] =
		tonic::include_file_descriptor_set!("ledger_descriptor");
}

pub fn get_router(database: &Pool<Postgres>) -> Result<Router, BackendError> {
	// Build ledger rpc server
	let utilities_server = UtilitiesServer::new(
		UtilitiesService::default()
	);

	let users_server = UsersServer::new(
		users::UsersService::new(database.clone())
	);

	let reflections_server = reflections::get_reflection()?;

	// Build RPC server router
	let router = Server::builder()
		.trace_fn(|_| tracing::info_span!("Tonic"))
		.add_service(reflections_server)
		.add_service(users_server)
		.add_service(utilities_server);

	Ok(router)
}
