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
    prelude::*, rpc::{self, get_router, proto::utilities_server::UtilitiesServer}, 
};

use crate::rpc::proto;
use crate::rpc::utilities::UtilitiesService;

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

        let router = rpc::get_router(&database)?;

        let address = format!(
            "{}:{}",
            settings.application.ip_address,
            settings.application.port
        );
        // We are using listener as it will bind a random port when port setting
        // is '0'. This is important for integration test server spawn.
        let listener = TcpListener::bind(address).await?;

        Ok(Self {
            database,
            router,
            listener,
        })
    }

    /// Run the Tonic server instance
    pub async fn run(self) -> Result<(), BackendError> {

        let address = format!(
            "{}:{}",
            self.listener.local_addr()?.ip(),
            self.listener.local_addr()?.port(),
        );
        tracing::info!("Tonic server started at '{}'", address);

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