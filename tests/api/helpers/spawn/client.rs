//-- ./tests/api/helpers/spawn/client.rs

// #![allow(unused)] // For beginning only.

/// Spawn a Tonic Client for testing server endpoints
///
/// #### Reference
///
/// * [Tonic LND client](https://github.com/Kixunil/tonic_lnd/blob/master/src/lib.rs)
/// ---

/// This is part of public interface, so it's re-exported.
pub extern crate tonic;

use authentication_service::domain;
use std::time;
use tonic::codegen::InterceptedService;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

/// Convenience type alias for authentication client.
// pub type AuthenticationClient =
// authentication_service::rpc::proto::authentication_service_client::AuthenticationServiceClient<
//     InterceptedService<Channel, TokenInterceptor>>;
pub type AuthenticationClient =
authentication_service::rpc::proto::authentication_service_client::AuthenticationServiceClient<Channel>;

/// Convenience type alias for sessions client
pub type SessionsClient =
authentication_service::rpc::proto::sessions_service_client::SessionsServiceClient<
    InterceptedService<Channel, TokenInterceptor>>;

// Convenience type alias for users client
pub type UsersClient =
    authentication_service::rpc::proto::users_service_client::UsersServiceClient<
        InterceptedService<Channel, TokenInterceptor>,
    >;

/// Tonic Client
#[derive(Clone)]
pub struct TonicClient {
    authentication: AuthenticationClient,
    sessions: SessionsClient,
    users: UsersClient,
}

impl TonicClient {
    /// Returns the authentication client.
    pub fn authentication(&mut self) -> &mut AuthenticationClient {
        &mut self.authentication
    }

    /// Returns the sessions client.
    pub fn sessions(&mut self) -> &mut SessionsClient {
        &mut self.sessions
    }

    /// Returns the users client.
    pub fn users(&mut self) -> &mut UsersClient {
        &mut self.users
    }

    /// Spawn a new tonic client based on the tonic server
    pub async fn spawn_client(
        server: &super::TonicServer,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Build Tonic Client channel
        let uri: tonic::transport::Uri = server.address.parse()?;

        // Create a channel builder with the server URI
        let mut endpoint = Channel::builder(uri);

        // Configure the endpoint tls if enabled. If the server is configured to use TLS, we need to load the TLS certificate and configure the endpoint to use TLS.
        if server.config.application.use_tls {
            // Load the server's certificate as CA
            let cert_path = server
                .config
                .application
                .tls_certificate
                .as_ref()
                .ok_or("TLS certificate path is not set in config")?;
            let cert_read = std::fs::read(cert_path)?;
            let cert = Certificate::from_pem(cert_read);

            let tls_config = ClientTlsConfig::new()
                .ca_certificate(cert)
                .domain_name("localhost"); // Adjust if your cert uses a different domain

            endpoint = endpoint.tls_config(tls_config)?;
        }

        // Add endpoint to the rpc channel
        let inner: Channel = endpoint.connect().await?;

        // Get tokens
        let access_token = server.clone().access_token;
        let refresh_token = server.clone().refresh_token;

        // Get refresh token duration
        let rt_duration = time::Duration::new(
            (server.config.application.refresh_token_duration_minutes * 60)
                .try_into()
                .unwrap(),
            0,
        );

        // Build refresh token as a string
        let refresh_cookie = refresh_token
            .build_cookie(&server.address, &rt_duration)
            .to_string();

        // Create client token interceptor
        let client_interceptor = TokenInterceptor {
            access_token,
            refresh_cookie,
        };

        // Build Authentication client request
        let authentication = AuthenticationClient::new(inner.clone());

        // Build sessions client request
        let sessions = authentication_service::rpc::proto::sessions_service_client::SessionsServiceClient::with_interceptor(inner.clone(), client_interceptor.clone());

        // Build Users client request
        let users = authentication_service::rpc::proto::users_service_client::UsersServiceClient::with_interceptor(inner.clone(), client_interceptor.clone());

        let client = TonicClient {
            authentication,
            sessions,
            users,
        };

        Ok(client)
    }
}

/// Supplies requests with access token
#[derive(Clone)]
pub struct TokenInterceptor {
    access_token: domain::AccessToken,
    refresh_cookie: String,
}

use http::header::COOKIE;
use http::HeaderMap;
use tonic::metadata::MetadataMap;

impl tonic::service::Interceptor for TokenInterceptor {
    #[tracing::instrument(name = "Token Interceptor: ", skip_all)]
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        // Create a new http header map
        let mut http_header = HeaderMap::new();
        // println!("Access Token: {:?}", self.access_cookie);
        // println!("Refresh Token: {:?}", self.refresh_cookie);

        // Add authorization bearer token to the http header
        http_header.append(
            "authorization",
            format!("Bearer {}", self.access_token.to_string())
                .parse()
                .unwrap(),
        );

        // Add refresh cookie to the http header
        http_header.append(COOKIE, self.refresh_cookie.parse().unwrap());

        // Add the http header to the rpc response
        *request.metadata_mut() = MetadataMap::from_headers(http_header);
        tracing::debug!("Added cookie headers to request: {:?}", request);

        Ok(request)
    }
}
