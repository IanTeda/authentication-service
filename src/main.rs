//-- ./src/main.rs

// #![allow(unused)] // For beginning only.

// use crate::prelude::*;

use tonic::{transport::Server, Request, Response, Status};

/// Namespace compiled Protobuf code
mod ledger {
    //-- Bring protobuf's into scope.
    tonic::include_proto!("ledger"); // The string specified here must match the proto package name

    pub(crate) const RPC_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("rpc_descriptor");
}

use ledger::rpc_server::{Rpc, RpcServer};
use ledger::{Empty, PongResponse};

mod error;
mod utils;

#[derive(Debug, Default)]
pub struct RpcService {}

#[tonic::async_trait]
impl Rpc for RpcService {
    async fn ping(
        &self,
        _request: Request<Empty> // Accept request of type HelloRequest
    ) -> Result<Response<PongResponse>, Status> { // Return an instance of type HelloReply

        let reply: PongResponse = PongResponse {
            message: "Pong...".to_string(),
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

/// Binary entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let rpc = RpcService::default();

    let reflections = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(ledger::RPC_DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        .add_service(reflections)
        .add_service(RpcServer::new(rpc))
        .serve(addr)
        .await?;

    Ok(())
}
