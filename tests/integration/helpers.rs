//-- ./tests/backend/helpers.rs

// #![allow(unused)] // For beginning only.

use personal_ledger_backend::{configuration::Settings, startup};

use sqlx::{Pool, Postgres};
use tonic::{metadata::MetadataValue, transport::{Channel, Uri}, Request, Status};

// Override test modules with more flexible error
pub type Error = Box<dyn std::error::Error>;
// pub type Result<T> = core::result::Result<T, Error>;

/// Remember the Test Server address so we can use it spawn the Test Client
pub struct TestServer {
	pub address: String,
}

/// Spawn a Tonic test server through the crates startup module
pub async fn spawn_test_server(database: Pool<Postgres>) -> Result<TestServer, Error> {
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

	// Give the test server a few ms to become available
	tokio::time::sleep(std::time::Duration::from_millis(100)).await;

	Ok(TestServer { address })
}

pub async fn get_client_channel(address: impl Into<String>) -> Result<Channel, Error> {
	let uri: Uri = address.into().parse()?;
	let endpoint = Channel::builder(uri);
	let channel = endpoint.connect().await?;

	Ok(channel)
}

/// This function will get called on each outbound request. Returning a
/// `Status` here will cancel the request and have that status returned to
/// the caller.
pub fn authentication_intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
	let token: MetadataValue<_> = "Bearer some-auth-token".parse().unwrap();
	req.metadata_mut().insert("authorization", token.clone());
	Ok(req)
}

