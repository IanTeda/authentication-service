//-- ./src/main.rs

// #![allow(unused)] // For beginning only.

use tonic::{transport::Server, Request, Response, Status};

/// Namespace compiled Protobuf code
mod ledger {
    //-- Bring protobuf's into scope.
    tonic::include_proto!("ledger"); // The string specified here must match the proto package name

    pub(crate) const DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("ledger_descriptor");
}

use ledger::rpc_server::{Rpc, RpcServer};
use ledger::{Empty, PongResponse};

mod configuration;
mod error;
mod prelude;
mod telemetry;
mod utils;

use crate::prelude::*;

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
async fn main() -> Result<(), BackendError> {

        // Parse configuration files
    let settings = configuration::Settings::parse()?;

    // Build tracing subscriber
    let tracing_subscriber = telemetry::get_tracing_subscriber(
        "personal_ledger_server".into(),
        std::io::stdout,
        settings.application.runtime_environment,
        settings.application.log_level
    );
    
    telemetry::init_tracing(tracing_subscriber)?;

    let address = format!(
        "{}:{}",
        settings.application.address,
        settings.application.port
    ).parse()?;

    let rpc_service = RpcService::default();

    let reflections_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(ledger::DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        .add_service(reflections_server)
        .add_service(RpcServer::new(rpc_service))
        .serve(address)
        .await?;

    Ok(())
}
