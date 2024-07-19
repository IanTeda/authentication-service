// -- ./src/rpc/mod.rs

//! RPC module containing endpoint configurations
//!
//! `proto` brings the Protobuf generated files into scope
//! `get_router` returns all the rpc endpoints for building the Tonic server.

// #![allow(unused)] // For development only

use std::sync::Arc;

use secrecy::Secret;
use sqlx::Pool;
use sqlx::Postgres;
use tonic::transport::{server::Router, Server};

use crate::configuration::Configuration;
use crate::middleware;
use crate::prelude::*;
use crate::reflections;
use crate::rpc::ledger::authentication_server::AuthenticationServer;
use crate::rpc::ledger::refresh_tokens_server::RefreshTokensServer;
use crate::rpc::ledger::users_server::UsersServer;
use crate::rpc::ledger::utilities_server::UtilitiesServer;
use crate::services::RefreshTokensService;
use crate::services::{AuthenticationService, UsersService, UtilitiesService};

// use crate::services::{AuthenticationService, UsersService, UtilitiesService};

pub fn get_router(
    database: Pool<Postgres>,
    config: Configuration,
) -> Result<Router, BackendError> {
    // Wraps our database pool in an Atomic Reference Counted pointer (Arc).
    // Each instance of the backend will get a pointer to the pool instead of getting a raw copy.
    let database = Arc::new(database);

    // Wrap config in an Atomic Reference Counted (ARC) pointer.
    let config = Arc::new(config);

    // Wrap token_secret string in a Secret
    let token_secret = Secret::new(config.application.token_secret.clone());

    // Intercept request and verify Access Token
    let access_token_interceptor =
        middleware::AccessTokenInterceptor { token_secret };

    // Build Utilities server
    let utilities_service = UtilitiesService::new(Arc::clone(&config));
    let utilities_server = UtilitiesServer::new(utilities_service);

    // Build Authentication server
    let authentication_service =
        AuthenticationService::new(Arc::clone(&database), Arc::clone(&config));
    let authentication_server = AuthenticationServer::new(authentication_service);

    // Build Users server
    let users_service =
        UsersService::new(Arc::clone(&database), Arc::clone(&config));
    let users_server = UsersServer::with_interceptor(
        users_service,
        access_token_interceptor.clone(),
    );

    // Build Refresh Tokens server
    let refresh_tokens_service =
        RefreshTokensService::new(Arc::clone(&database), Arc::clone(&config));
    let refresh_tokens_server = RefreshTokensServer::with_interceptor(
        refresh_tokens_service,
        access_token_interceptor,
    );

    // Build reflections server
    let reflections_server = reflections::get_reflection()?;

    // Build RPC server router
    let router = Server::builder()
        .trace_fn(|_| tracing::info_span!("Tonic"))
        .add_service(reflections_server)
        .add_service(authentication_server)
        .add_service(users_server)
        .add_service(utilities_server)
        .add_service(refresh_tokens_server);

    Ok(router)
}
