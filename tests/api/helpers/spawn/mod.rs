#![allow(unused)] // For beginning only.

use std::sync::Arc;

use once_cell::sync::Lazy;
use secrecy::Secret;
use sqlx::{Pool, Postgres};
use tonic::{
	metadata::MetadataValue,
	Request,
	Status, transport::{Channel, Uri},
};

use personal_ledger_backend::{configuration::Configuration, domain, startup, telemetry};
use personal_ledger_backend::configuration::{Environment, LogLevels};

use super::mocks;
mod client;
pub use client::TonicClient;

pub type Error = Box<dyn std::error::Error>;


// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = LogLevels::Info;
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let tracing_subscriber = telemetry::get_tracing_subscriber(
            subscriber_name,
            std::io::stdout,
            Environment::Development,
            default_filter_level,
        );
        let _ = telemetry::init_tracing(tracing_subscriber);
    } else {
        let subscriber = telemetry::get_tracing_subscriber(
            subscriber_name,
            std::io::sink,
            Environment::Development,
            default_filter_level,
        );
        let _ = telemetry::init_tracing(subscriber);
    };
});

#[derive(Clone)]
pub struct TonicServer {
    pub address: String,
    pub access_token: String,
    pub config: Arc<Configuration>,
}

impl TonicServer {
    pub async fn spawn_server(database: Pool<Postgres>) -> Result<Self, Error> {
        // Initiate tracing in integration testing
        Lazy::force(&TRACING);

        // Parse configuration files
        let config = {
            let mut s = Configuration::parse()?;
            // Change port to `0` to avoid conflicts as the OS will assign an unused port
            s.application.port = 0;
            s
        };

        // Generate random user data for testing and insert in test database
        let random_password = mocks::password()?; // In case we need it in the future
        let random_user = mocks::user_model(&random_password)?;
        let _database_record = random_user.insert(&database).await?;

        // Build Tonic server using startup
        let tonic_server = startup::TonicServer::build(config.clone(), database).await?;

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

        // Generate access token for Tonic Client requests
        let token_secret = &config.application.token_secret;
        let token_secret = Secret::new(token_secret.to_owned());
        let access_token_string = domain::AccessToken::new(&token_secret, &random_user.id).await?.to_string();
        // let access_token = mocks::access_token(&random_user.id, &token_secret).await?.to_string();

        let config = Arc::new(config);

        // unimplemented!()
        Ok(Self { access_token: access_token_string, address, config })
    }

    pub async fn client_channel(self) -> Result<Channel, Error> {
        let uri: Uri = self.address.parse()?;
        let endpoint = Channel::builder(uri);
        let channel = endpoint.connect().await?;

        // unimplemented!()
        Ok(channel)
    }

    pub async fn authenticate() {}
}

/// This function will get called on each outbound request. Returning a
/// `Status` here will cancel the request and have that status returned to
/// the caller.
pub fn authentication_intercept(mut req: Request<()>, token: String) -> Result<Request<()>, Status> {
    println!("Intercepting request: {:?}", req);
    let token: MetadataValue<_> = token.parse().unwrap();
    req.metadata_mut().insert("authorization", token.clone());
    Ok(req)
}
