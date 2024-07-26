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
use crate::rpc::proto::authentication_server::AuthenticationServer;
use crate::rpc::proto::logins_server::LoginsServer;
use crate::rpc::proto::refresh_tokens_server::RefreshTokensServer;
use crate::rpc::proto::users_server::UsersServer;
use crate::rpc::proto::utilities_server::UtilitiesServer;
use crate::services;

// use crate::services::{AuthenticationService, UsersService, UtilitiesService};

pub fn get_router(
    database: Pool<Postgres>,
    config: Configuration,
) -> Result<Router, BackendError> {
    // Wraps our database pool in an Atomic Reference Counted (ARC).
    // Each instance of the backend will get a pointer to the pool instead of getting a raw copy.
    let database = Arc::new(database);

    // Wrap config in an Atomic Reference Counted (ARC).
    let config = Arc::new(config);

    // Wrap token_secret string in a Secret
    let token_secret = Secret::new(config.application.token_secret.clone());

    // Intercept request and verify Access Token
    let access_token_interceptor =
        middleware::AccessTokenInterceptor { token_secret };

    // Build Utilities server
    let utilities_service = services::UtilitiesService::new(Arc::clone(&config));
    
    let utilities_server = UtilitiesServer::new(utilities_service);

    // Build Authentication server
    let authentication_service =
        services::AuthenticationService::new(Arc::clone(&database), Arc::clone(&config));
    
    let authentication_server = AuthenticationServer::new(authentication_service);

    // Build Users server
    let users_service =
        services::UsersService::new(Arc::clone(&database), Arc::clone(&config));
    
    let users_server = UsersServer::with_interceptor(
        users_service,
        access_token_interceptor.clone(),
    );

    // Build Refresh Tokens server
    let refresh_tokens_service =
        services::RefreshTokensService::new(Arc::clone(&database), Arc::clone(&config));
    
    let refresh_tokens_server = RefreshTokensServer::with_interceptor(
        refresh_tokens_service,
        access_token_interceptor.clone(),
    );

    // Build Logins Tokens server
    let logins_service =
        services::LoginsService::new(Arc::clone(&database), Arc::clone(&config));

    let logins_server = LoginsServer::with_interceptor(
        logins_service,
        access_token_interceptor,
    );

    // Build reflections server
    let reflections_server = services::ReflectionsService::new();

    // Build RPC server router
    let router = Server::builder()
        .trace_fn(|_| tracing::info_span!("Tonic"))
        .add_service(reflections_server)
        .add_service(utilities_server)
        .add_service(authentication_server)
        .add_service(users_server)
        .add_service(refresh_tokens_server)
        .add_service(logins_server);

    Ok(router)
}
