// -- ./src/router.rs

//! # GRPC Router
//!
//! RPC module containing endpoint configurations
//!
//! `proto` brings the Protobuf generated files into scope
//! `get_router` returns all the rpc endpoints for building the Tonic server.
//! 
//! ## Install
//! 
//! To get localhost https certificates we need to do a few things
//! 
//! 1. Install MkCert
//! 
//! ```bash
//! sudo apt install mkcert libnss3-tools
//! ```
//! 
//! 2. Install local Certificate Authority (CA)
//! 
//! ```bash
//! mkcert --install
//! ```
//! 
//! 3. Generate certificates
//! 
//! Need a server key and pem
//! 
//! ```bash
//! cd tls
//! mkcert server
//! ```
//! 
//! ## Actions and fixes
//! 
//! - [ ] Add cert creation to the Dockerfile
//! - [ ] Should this be made into a struct
//!
//! ## References
//!
//! - https://github.com/nicktretyakov/gRUSTpcWEB

// #![allow(unused)] // For development only

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
const DEFAULT_ALLOW_HEADERS: [&str; 4] =
    ["x-grpc-web", "content-type", "x-user-agent", "grpc-timeout"];

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
) -> Result<
    tonic_transport::server::Router<
        tower_layer::Stack<
            tonic_web::GrpcWebLayer,
            tower_layer::Stack<cors::CorsLayer, tower_layer::Identity>,
        >,
    >,
    AuthenticationError,
> {
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

    // TODO: Set up TLS
    // let tls_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR"), "tls"]);
    // let cert = std::fs::read_to_string(tls_dir.join("server.pem"))?;
    // let key = std::fs::read_to_string(tls_dir.join("server-key.pem"))?;
    // let identity = tonic_transport::Identity::from_pem(cert, key);


    let router = tonic_transport::Server::builder()
        // Start tonic log tracing
        .trace_fn(|_| tracing::info_span!("Tonic"))
        // .tls_config(tonic_transport::ServerTlsConfig::new().identity(identity))?
        // Enable http/1.1 support. GRPC-web requires http/1.1.
        .accept_http1(true)
        // Add the cors layer
        .layer(cors_layer)
        // Add a single GrpcWebLayer for all services
        .layer(tonic_web::GrpcWebLayer::new()) // Add the gRPC-Web layer
        // Add services
        .add_service(rpc::spec_service()?)
        .add_service(utilities_server)
        .add_service(authentication_server)
        .add_service(users_server)
        .add_service(sessions_server);

    Ok(router)
}