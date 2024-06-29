// -- ./startup.rs

#![allow(unused)] // For beginning only.

//! Helper functions for starting the Tonic server.
//! 
//! # Startup
//! 
//! This module has a Tonic Server instance enum for reuse in the integration
//! test suit.
//! ---

use crate::{
    configuration::{DatabaseSettings, Settings},
    prelude::*, 
    rpc::{ledger::{self, rpc_server::RpcServer}, LedgerService}
};

use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};
use tokio::net::TcpListener;
use tonic::transport::{server::Router, Server};

/// Tonic Server instance enum;
pub struct TonicServer {
    pub database: PgPool,
    pub router: Router,
    pub listener: TcpListener,
}


impl TonicServer {
    /// Build the Tonic server instance.
    pub async fn build(settings: Settings, database: Pool<Postgres>)
    -> Result<Self, BackendError> {

        let database = database;

        let address = format!(
            "{}:{}",
            settings.application.ip_address,
            settings.application.port
        );
        // We are using listener as it will bind a random port when port setting
        // is '0'. This is important for integration test server spawn.
        let listener = TcpListener::bind(address).await?;
        
        // Build reflection server
        let reflections_server = 
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(ledger::DESCRIPTOR_SET)
                .build()?;

        // Build ledger rpc server
        let ledger_server = RpcServer::new(
            LedgerService::default()
        );

        // Build RPC server router
        let router = Server::builder()
            .add_service(reflections_server)
            .add_service(ledger_server);

        Ok(Self {
            database,
            router,
            listener,
        })
    }

    /// Run the Tonic server instance
    pub async fn run(self) -> Result<(), BackendError> {
        let incoming = tokio_stream::wrappers::TcpListenerStream::new(self.listener);
        self.router.serve_with_incoming(incoming).await?;
        Ok(())
    }
}

pub async fn get_database(
    database: &DatabaseSettings
) -> Result<PgPool, BackendError> {
    // Build connection pool
	let database =
		PgPoolOptions::new()
            .connect_lazy_with(database.connection());

    sqlx::migrate!("./migrations")
		.run(&database)
		.await?;

    Ok(database)
}
