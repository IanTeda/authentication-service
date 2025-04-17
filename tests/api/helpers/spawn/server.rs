//-- ./tests/api/helpers/spawn/server.rs

// #![allow(unused)] // For beginning only.

/// Spawn a Tonic Server for testing endpoints
///
/// Server is spun up using the main crate startup function, so we use the same
/// code as the crate
/// ---
use std::sync::Arc;

use authentication_service::{
    configuration::Configuration, domain, startup, telemetry,
};
use once_cell::sync::Lazy;
use sqlx::{Pool, Postgres};
use std::time;
use tonic::transport::{Channel, Uri};
use tracing::level_filters::LevelFilter;

use crate::helpers::mocks;

pub type Error = Box<dyn std::error::Error>;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
// Lazy makes it globally available
static TRACING: Lazy<()> = Lazy::new(|| {
    let testing_log_level = LevelFilter::ERROR;
    let _telemetry = telemetry::init(testing_log_level);
});

#[derive(Clone)]
pub struct TonicServer {
    pub address: String,
    pub access_token: domain::AccessToken,
    pub refresh_token: domain::RefreshToken,
    pub config: Arc<Configuration>,
}

impl TonicServer {
    pub async fn spawn_server(database: &Pool<Postgres>) -> Result<Self, Error> {
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
        let mut random_user = mocks::users(&random_password)?;
        random_user.is_active = true;
        random_user.is_verified = true;
        random_user.role = domain::UserRole::Admin;
        let random_user = random_user.insert(&database).await?;
        tracing::debug!("Random User: {:?}", random_user);

        // Get the token issuer from the configuration
        let issuer = config.application.get_issuer();

        // Get the token secret from the configuration
        let token_secret = &config.application.token_secret;

        // Generate refresh token for Tonic Client requests
        let rt_duration = time::Duration::new(
            (&config.application.refresh_token_duration_minutes * 60)
                .try_into()
                .unwrap(),
            0,
        );
        let refresh_token = domain::RefreshToken::new(
            &token_secret,
            &issuer,
            &rt_duration,
            &random_user,
        )?;
        tracing::debug!("Refresh token: {}", refresh_token);

        // Generate session login to get refresh token
        let mut session = mocks::sessions(&random_user, &refresh_token)?;
        session.is_active = true;
        let _database_session = session.insert(&database).await?;
        tracing::debug!("Session: {:?}", session);

        // Generate access token for Tonic Client requests
        let at_duration = time::Duration::new(
            (&config.application.access_token_duration_minutes * 60)
                .try_into()
                .unwrap(),
            0,
        );
        let access_token = domain::AccessToken::new(
            &token_secret,
            &issuer,
            &at_duration,
            &random_user,
        )?;
        tracing::debug!("Access token: {}", access_token);

        // Build Tonic server using main crate startup
        let tonic_server =
            startup::TonicServer::build(config.clone(), database.clone()).await?;

        // Set tonic server address as the port is randomly selected by the TCP Listener (in startup)
        // when config sets the port to 0
        let address = format!(
            "http://{}:{}",
            // tonic_server.listener.local_addr()?.ip(),
            config.application.ip_address,
            tonic_server.listener.local_addr()?.port()
        );
        tracing::debug!("Tonic test server at: {:?}", address);

        // Run as a background task by wrapping server instance in a tokio future
        tokio::spawn(async move {
            let _ = tonic_server.run().await;
        });

        // Give the test server a few ms to become available
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let config = Arc::new(config);

        Ok(Self {
            address,
            access_token,
            refresh_token: session.refresh_token,
            config,
        })
    }

    pub async fn client_channel(self) -> Result<Channel, Error> {
        let uri: Uri = self.address.parse()?;
        let endpoint = Channel::builder(uri);
        let channel = endpoint.connect().await?;

        Ok(channel)
    }

    pub async fn authenticate() {}
}
