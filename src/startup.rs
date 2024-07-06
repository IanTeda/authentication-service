// -- ./startup.rs

// #![allow(unused)] // For beginning only.

//! Helper functions for starting the Tonic server.
//!
//! # Startup
//!
//! This module has a Tonic Server instance enum for reuse in the integration
//! test suit.
//! ---

use crate::{configuration::Configuration, prelude::*, router};

use sqlx::{Pool, Postgres};
use tokio::net::TcpListener;
use tonic::transport::server::Router;

/// Tonic Server instance enum;
pub struct TonicServer {
	pub router: Router,
	pub listener: TcpListener,
}

impl TonicServer {
	/// Build the Tonic server instance.
	pub async fn build(
		configuration: Configuration,
		database: Pool<Postgres>,
	) -> Result<Self, BackendError> {
		let router = router::get_router(database, configuration.application.jwt_secret.as_bytes())?;

		let address = format!(
			"{}:{}",
			configuration.application.ip_address, configuration.application.port
		);
		// We are using listener as it will bind a random port when port setting
		// is '0'. This is important for integration test server spawn.
		let listener = TcpListener::bind(address).await?;

		Ok(Self { router, listener })
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
