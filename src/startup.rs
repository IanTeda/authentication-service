// -- ./startup.rs

#![allow(unused)] // For beginning only.


//! A helper function for starting the Tonic server.
//! ---

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tonic::transport::{server::Router, Server};

use crate::{
    configuration::Settings,
    grpc::{
        ledger::{self, rpc_server::RpcServer, DESCRIPTOR_SET}, RpcService
    },
    prelude::*
};

/// Application port and server instance
pub struct Application {
	port: u16,
	server: Router,
}

impl Application {
    pub async fn build(settings: Settings)
    -> Result<Self, BackendError> {
        let address = format!("{}:{}",
            settings.application.ip_address,
            settings.application.port
        );
        let socket: SocketAddr = address.parse()?;

	    let rpc_service = RpcService::default();

        let reflections_server = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(ledger::DESCRIPTOR_SET)
            .build()?;

        let server = Server::builder()
            .add_service(reflections_server)
            .add_service(RpcServer::new(rpc_service));

        let port = socket.port();

        Ok(Self {
            port,
            server
        })

    }

}

