// -- ./startup.rs

// #![allow(unused)] // For beginning only.


//! A helper function for starting the Tonic server.
//! ---

use std::net::SocketAddr;

// use sqlx::PgPool;
use tonic::transport::{server::Router, Server};

use crate::{
    configuration::Settings,
    prelude::*, rpc::{ledger::{self, rpc_server::RpcServer}, RpcService}
};

/// Application port and server instance
pub struct TonicServer {
	// port: u16,
    // database: PgPool,
    socket: SocketAddr,
    router: Router
}

impl TonicServer {
    /// Build the Tonic server instance.
    pub async fn build(settings: Settings)
    -> Result<Self, BackendError> {
        let address = format!("{}:{}",
            settings.application.ip_address,
            settings.application.port
        );
        let socket: SocketAddr = address.parse()?;

	    let rpc_service: RpcService = RpcService::default();

        let reflections_server = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(ledger::DESCRIPTOR_SET)
            .build()?;

        let router = Server::builder()
            .add_service(reflections_server)
            .add_service(RpcServer::new(rpc_service));

        // let port = socket.port();

        Ok(Self {
            socket,
            router
        })
    }

    /// Run the Tonic server instance
    pub async fn run(self) -> Result<(), BackendError> {
        self.router.serve(self.socket).await?;
        Ok(())
    }
}

