//-- ./tests/backend/helpers.rs

// #![allow(unused)] // For beginning only.

use personal_ledger_backend::{configuration::Settings, startup};
use personal_ledger_backend::rpc::ledger::rpc_client::RpcClient;
use tonic::transport::Channel;

use sqlx::{Pool, Postgres};

// Override test modules with more flexible error
pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

/// Remember the Test Server address so we can use it spawn the Test Client
pub struct TestServer {
	pub address: String,
}

/// Spawn a Tonic test server through the crates startup module
pub async fn spawn_test_server(database: Pool<Postgres>) -> Result<TestServer> {
	// Parse configuration files
	let settings = {
		let mut s = Settings::parse()?;
		// Set port to `0` avoids conflicts as the OS will assign an unused port
		s.application.port = 0;
		s
	};

	// Build Tonic server using main crate startup
	let tonic_server = startup::TonicServer::build(settings, database)
		.await?;

	// Set tonic server address as the port is randomly selected to TCP Listener
	// in startup::TonicServer
	let address = format!(
		"http://{}:{}",
		tonic_server.listener.local_addr()?.ip(),
		tonic_server.listener.local_addr()?.port()
	);

	// Run as a background task by wrapping server instance in a tokio future
	tokio::spawn(async move {
		let _ = tonic_server.run().await;
	});

	Ok(TestServer { address })
}

/// Spawn a test client
pub async fn spawn_test_client(address: impl Into<String>) -> Result<RpcClient<Channel>> {
	let client = RpcClient::connect(address.into()).await?;

	Ok(client)
}

