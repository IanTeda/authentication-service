//-- ./src/rpc/utilities.rs

//! Return a result containing a RPC Utilities server

// #![allow(unused)] // For beginning only.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::rpc::ledger::utilities_server::Utilities;
use crate::rpc::ledger::{Empty, PingResponse};
use crate::utilities::jwt::JwtKeys;

// #[derive(Debug, Default)]
pub struct UtilitiesService {
	#[allow(dead_code)]
	jwt_keys: Arc<JwtKeys>,
}

impl UtilitiesService {
	pub fn new(jwt_keys: Arc<JwtKeys>) -> Self {
		Self { jwt_keys }
	}
}

#[tonic::async_trait]
impl Utilities for UtilitiesService {
	#[tracing::instrument(
		name = "Ping endpoint",
		skip(self),
		// fields(
		// 	Request = %_request
		// ),
	)]
	async fn ping(&self, _request: Request<Empty>) -> Result<Response<PingResponse>, Status> {
		let reply = PingResponse {
			message: "Pong...".to_string(),
		};

		// Send back our ping response.
		Ok(Response::new(reply))
	}
}
