// -- ./src/grpc.rs

//! Application configuration settings
//!
//! # Application Configuration Crate
//!
//! Get API configuration from the `./configuration/base.yaml` file and
//! overwrite with runtime environment configuration `./config/production.yaml`
//! and environmental runtime variables.
//!
//! # References
//!
//! * [config.rs Repository](https://github.com/mehcode/config-rs)
//! * [Configuration management in Rust web services](https://blog.logrocket.com/configuration-management-in-rust-web-services/)

#![allow(unused)] // For development only

use crate::prelude::*;

use tonic::{transport::Server, Request, Response, Status};

pub mod ledger {
	tonic::include_proto!("ledger");
	pub(crate) const DESCRIPTOR_SET: &[u8] =
		tonic::include_file_descriptor_set!("ledger_descriptor");
}

use ledger::rpc_server::{Rpc, RpcServer};
use ledger::{Empty, PongResponse};

#[derive(Debug, Default)]
pub struct RpcService {}

#[tonic::async_trait]
impl Rpc for RpcService {
	async fn ping(
		&self,
		_request: Request<Empty>, // Accept request of type HelloRequest
	) -> Result<Response<PongResponse>, Status> {
		// Return an instance of type HelloReply

		let reply: PongResponse = PongResponse {
			message: "Pong...".to_string(),
		};

		Ok(Response::new(reply)) // Send back our formatted greeting
	}
}
