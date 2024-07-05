

// #![allow(unused)] // For beginning only.

use sqlx::{Pool, Postgres};

use personal_ledger_backend::{configuration::Settings, startup};
use tonic::{metadata::MetadataValue, transport::{Channel, Uri}, Request, Status};

pub type Error = Box<dyn std::error::Error>;

pub struct TonicServer {
    pub address: String,
}

impl TonicServer {
    pub async fn spawn_server(database: Pool<Postgres>) -> Result<Self, Error> {
        // Parse configuration files
        let settings = {
            let mut s = Settings::parse()?;
            // Change port to `0` to avoid conflicts as the OS will assign an unused port
            s.application.port = 0;
            s
        };

        // Build Tonic server using startup
        let tonic_server = startup::TonicServer::build(settings, database)
            .await?;

        // Set tonic server address as the port is randomly selected by the TCP Listener (in startup)
        // when config sets the port to 0
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

        // unimplemented!()
        Ok(Self { address })
    }

    pub async fn client_channel(self) -> Result<Channel, Error>{
        let uri: Uri = self.address.parse()?;
	    let endpoint = Channel::builder(uri);
	    let channel = endpoint.connect().await?;

        // unimplemented!()
        Ok(channel)
    }
}

/// This function will get called on each outbound request. Returning a
/// `Status` here will cancel the request and have that status returned to
/// the caller.
pub fn authentication_intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
	let token: MetadataValue<_> = "Bearer some-auth-token".parse().unwrap();
	req.metadata_mut().insert("authorization", token.clone());
	Ok(req)
}