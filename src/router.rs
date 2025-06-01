// -- ./src/router.rs

// #![allow(unused)] // For development only

//! # GRPC Router
//!
//! This module configures and builds the gRPC server for the authentication service using Tonic.
//!
//! It sets up all RPC endpoints, middleware (including authorization), and CORS for gRPC-Web support.
//! TLS/HTTPS is supported and can be enabled via configuration, with certificate and key paths configurable.
//!
//! ## Local Development: TLS Certificates
//!
//! To enable HTTPS/TLS for local development, generate self-signed certificates using [mkcert](https://github.com/FiloSottile/mkcert):
//!
//! 1. **Install MkCert**
//!    ```bash
//!    sudo apt install mkcert libnss3-tools
//!    ```
//!
//! 2. **Install local Certificate Authority (CA)**
//!    ```bash
//!    mkcert --install
//!    ```
//!
//! 3. **Generate certificates**
//!    ```bash
//!    cargo make generate_tls
//!    ```
//!    This will create `tls/server.pem` and `tls/server-key.pem`.
//!
//! ## Configuration
//!
//! - Set `use_tls = true` in your configuration to enable TLS.
//! - Set `tls_certificate` and `tls_private_key` to the paths of your certificate and key files.
//!
//! ## Actions and Fixes
//!
//! - [ ] Add cert creation to the Dockerfile
//! - [ ] Consider refactoring into a struct for improved testability and modularity
//!
//! ## References
//!
//! - https://github.com/nicktretyakov/gRUSTpcWEB
//! - https://github.com/hyperium/tonic

use std::sync::Arc;

use core::time;
use http::HeaderName;
use sqlx::Pool;
use sqlx::Postgres;
use tonic::transport as tonic_transport;
use tower_http::cors;

use crate::configuration::Configuration;
use crate::domain;
use crate::middleware;
use crate::prelude::*;
use crate::rpc;
use crate::rpc::proto::authentication_service_server::AuthenticationServiceServer as AuthenticationServer;
use crate::rpc::proto::sessions_service_server::SessionsServiceServer as SessionsServer;
use crate::rpc::proto::users_service_server::UsersServiceServer as UsersServer;
use crate::rpc::proto::utilities_service_server::UtilitiesServiceServer as UtilitiesServer;
use crate::services;

//-- Constants
// Default max age for CORS preflight requests
// This is the time the browser will cache the preflight response
// before sending a new preflight request.
const DEFAULT_MAX_AGE: time::Duration = time::Duration::from_secs(24 * 60 * 60);

// Default exposed headers for CORS
// These are the headers that will be exposed to the browser
// when the response is returned from the server.
// The browser will not expose these headers by default
// unless they are specified in the CORS response.
// The gRPC-web client will use these headers to parse the response.
const DEFAULT_EXPOSED_HEADERS: [&str; 3] =
    ["grpc-status", "grpc-message", "grpc-status-details-bin"];

// Default allowed headers for CORS
// These are the headers that will be allowed in the CORS request
// The browser will not send these headers by default
// unless they are specified in the CORS request.
// The gRPC-web client will use these headers to send the request.
const DEFAULT_ALLOW_HEADERS: [&str; 5] = [
    "x-grpc-web",
    "content-type",
    "x-user-agent",
    "grpc-timeout",
    "authorization",
];

// Use a type alias for the gRPC router for cleaner code and easier reference
type GrpcRouter = tonic_transport::server::Router<
    tower_layer::Stack<
        tonic_web::GrpcWebLayer,
        tower_layer::Stack<cors::CorsLayer, tower_layer::Identity>,
    >,
>;

/// # GRPC Router
///
/// RPC module containing endpoint configurations
///
/// `database: Pool<Postgres>` - The database connection pool
/// `config: Configuration` - The application configuration
///
/// ## References
///
pub fn get_router(
    database: Pool<Postgres>,
    config: Configuration,
) -> Result<GrpcRouter, AuthenticationError> {
    // Wraps our database pool and config in an Atomic Reference Counted (ARC).
    // Each instance of the backend will get a pointer to the pool instead of getting a raw copy.
    let database = Arc::new(database);
    let config = Arc::new(config);

    // Get the token secret and issuer from the config
    let token_secret = config.application.token_secret.clone();
    let issuer = config.application.get_issuer();

    // Create the interceptor
    // let access_token_interceptor = middleware::AccessTokenInterceptor {
    //     token_secret,
    //     issuer,
    // };

    // Build CORS layer
    let cors_layer = tower_http::cors::CorsLayer::new()
        .allow_origin(cors::AllowOrigin::mirror_request())
        .allow_credentials(true)
        .max_age(DEFAULT_MAX_AGE)
        .expose_headers(
            DEFAULT_EXPOSED_HEADERS
                .iter()
                .cloned()
                .map(HeaderName::from_static)
                .collect::<Vec<HeaderName>>(),
        )
        .allow_headers(
            DEFAULT_ALLOW_HEADERS
                .iter()
                .cloned()
                .map(HeaderName::from_static)
                .collect::<Vec<HeaderName>>(),
        );

    //-- Build the Utilities Service
    // Create a new UtilitiesService instance
    let utilities_service = services::UtilitiesService::new(Arc::clone(&config));

    // Wrap the UtilitiesService in the UtilitiesServiceServer
    let utilities_server = UtilitiesServer::new(utilities_service);

    //-- Build the Authentication Service
    // Create a new AuthenticationService instance
    let authentication_service = services::AuthenticationService::new(
        Arc::clone(&database),
        Arc::clone(&config),
    );

    // Wrap the AuthenticationService in the AuthenticationServiceServer
    let authentication_server = AuthenticationServer::new(authentication_service);

    //-- Build the Users Service
    // Create a new UsersService instance
    let users_service =
        services::UsersService::new(Arc::clone(&database), Arc::clone(&config));

    // Wrap the UsersService in the UsersServiceServer
    // let users_server = UsersServer::new(users_service); // <-- For testing with no access token
    let users_server = UsersServer::with_interceptor(
        users_service,
        middleware::AuthorisationInterceptor {
            token_secret: token_secret.clone(),
            issuer: issuer.clone(),
            allowable_roles: vec![domain::UserRole::Admin, domain::UserRole::User],
        },
    );

    //-- Build the Sessions Service
    // Create a new SessionsService instance
    let sessions_service =
        services::SessionsService::new(Arc::clone(&database), Arc::clone(&config));

    // Wrap the SessionsService in the SessionsServiceServer
    let sessions_server = SessionsServer::with_interceptor(
        sessions_service,
        middleware::AuthorisationInterceptor {
            token_secret: token_secret.clone(),
            issuer: issuer.clone(),
            allowable_roles: vec![domain::UserRole::Admin, domain::UserRole::User],
        },
    );

    //-- Build the Tonic Router

    // Create a new Tonic server builder. The Tonic server builder is used to configure
    // the server. It allows us to add services, middlewares, and other configurations.
    // The server builder is used to create the Tonic server
    let mut server_builder = tonic_transport::Server::builder()
        .trace_fn(|_| tracing::info_span!("Tonic"))
        .accept_http1(true)
        .layer(cors_layer)
        .layer(tonic_web::GrpcWebLayer::new());

    // If the application is configured to use TLS, we need to load the TLS identity
    // and configure the server to use TLS.
    if config.application.use_tls {
        // Load the TLS certificate and private key from the configuration
        // If the paths are not set in the configuration, we return an error.
        let cert_path =
            config.application.tls_certificate.as_ref().ok_or_else(|| {
                AuthenticationError::Generic(
                    "TLS certificate path is not set in configuration".to_string(),
                )
            })?;
        let cert = std::fs::read(cert_path).map_err(|e| {
            AuthenticationError::Generic(format!(
                "Failed to read server certificate at tls/server.pem: {e}"
            ))
        })?;
        let key_path =
            config.application.tls_private_key.as_ref().ok_or_else(|| {
                AuthenticationError::Generic(
                    "TLS private key path is not set in configuration".to_string(),
                )
            })?;
        let key = std::fs::read(key_path).map_err(|e| {
            AuthenticationError::Generic(format!(
                "Failed to read private key at tls/server-key.pem: {e}"
            ))
        })?;

        // Create a TLS identity from the certificate and private key
        let identity = tonic::transport::Identity::from_pem(cert, key);

        // Configure the server builder to use TLS with the identity. The identity
        // is used to encrypt the communication between the client and server.
        // This is required for gRPC over TLS.
        server_builder = server_builder.tls_config(
            tonic_transport::ServerTlsConfig::new().identity(identity),
        )?;
    }

    // Add the services to the server builder. The services are added to the server
    // builder, which will be used to create the Tonic server.
    let router = server_builder
        .add_service(rpc::spec_service()?)
        .add_service(utilities_server)
        .add_service(authentication_server)
        .add_service(users_server)
        .add_service(sessions_server);

    Ok(router)
}
