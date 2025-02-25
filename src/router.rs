// -- ./src/rpc/mod.rs

//! RPC module containing endpoint configurations
//!
//! `proto` brings the Protobuf generated files into scope
//! `get_router` returns all the rpc endpoints for building the Tonic server.

// #![allow(unused)] // For development only

use std::sync::Arc;

use sqlx::Pool;
use sqlx::Postgres;
use tonic::transport::server::Router;
use tonic::transport::Server;

use crate::configuration::Configuration;
use crate::middleware;
use crate::prelude::*;
use crate::rpc::proto::authentication_service_server::AuthenticationServiceServer as AuthenticationServer;
use crate::rpc::proto::logins_service_server::LoginsServiceServer as LoginsServer;
use crate::rpc::proto::sessions_service_server::SessionsServiceServer as SessionsServer;
use crate::rpc::proto::users_service_server::UsersServiceServer as UsersServer;
use crate::rpc::proto::utilities_service_server::UtilitiesServiceServer as UtilitiesServer;
use crate::services;

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
    let token_secret = config.application.token_secret.clone();

    // Intercept request and verify Access Token
    let access_token_interceptor =
        middleware::AccessTokenInterceptor { token_secret };

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
        access_token_interceptor.clone(),
    );

    //-- Build the Sessions Service
    // Create a new SessionsService instance
    let sessions_service =
        services::SessionsService::new(Arc::clone(&database), Arc::clone(&config));

    // Wrap the SessionsService in the SessionsServiceServer
    let sessions_server = SessionsServer::with_interceptor(
        sessions_service,
        access_token_interceptor.clone(),
    );

    //-- Build the Logins Service
    // Create a new LoginsService instance
    let logins_service =
        services::LoginsService::new(Arc::clone(&database), Arc::clone(&config));

    // Wrap the LoginsService in the LoginsServiceServer
    let logins_server =
        LoginsServer::with_interceptor(logins_service, access_token_interceptor);

    // Build reflections server
    // let reflections_server = services::ReflectionsService::new();

    // https://github.com/nicktretyakov/gRUSTpcWEB

    let router = Server::builder()
        // Start log tracing
        .trace_fn(|_| tracing::info_span!("Tonic"))
        // GRPC-web requires http/1.1
        .accept_http1(true)
        // Add reflection service
        // .add_service(reflections_server)
        // .add_service(utilities_server)
        .add_service(tonic_web::enable(utilities_server))
        .add_service(tonic_web::enable(authentication_server))
        .add_service(tonic_web::enable(users_server))
        .add_service(tonic_web::enable(sessions_server))
        .add_service(tonic_web::enable(logins_server));

    Ok(router)
}
