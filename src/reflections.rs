//-- ./src/rpc/reflections.rs

//! Return a result containing a RPC reflections server

// #![allow(unused)] // For beginning only.

use tonic_reflection::server::{ServerReflection, ServerReflectionServer};

use crate::prelude::*;

// Bring Protobuf generated files into scope
mod rpc {
	pub(crate) const DESCRIPTOR_SET: &[u8] =
		tonic::include_file_descriptor_set!("ledger_descriptor");
}

pub fn get_reflection(
) -> Result<ServerReflectionServer<impl ServerReflection>, BackendError> {
	let reflections_server = tonic_reflection::server::Builder::configure()
		.register_encoded_file_descriptor_set(rpc::DESCRIPTOR_SET)
		.build()?;

	Ok(reflections_server)
}
