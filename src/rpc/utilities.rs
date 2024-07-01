//-- ./src/rpc/utilities.rs

//! Return a result containing a RPC Utilities server

// #![allow(unused)] // For beginning only.

use tonic::{Request, Response, Status};

use super::proto::utilities_server::Utilities;
use super::proto::{Empty, PingResponse};

#[derive(Debug, Default)]
pub struct UtilitiesService {}

#[tonic::async_trait]
impl Utilities for UtilitiesService {

	#[tracing::instrument(
		name = "Ping endpoint",
		skip(self),
		// fields(
		// 	Request = %_request
		// ),
	)]
	async fn ping(
		&self,
		request: Request<Empty>,
	) -> Result<Response<PingResponse>, Status> {

		let reply = PingResponse {
			message: "Pong...".to_string(),
		};

		// Send back our ping response.
		Ok(Response::new(reply)) 
	}
}