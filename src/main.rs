//-- ./src/main.rs

// #![allow(unused)] // For beginning only.

// For intellisense
mod configuration;
mod database;
mod domain;
mod error;
mod middleware;
mod prelude;
mod router;
mod rpc;
mod services;
mod startup;
mod telemetry;
mod utils;

// use configuration::Configuration;

use configuration::Configuration;

use crate::prelude::*;

/// Binary entry point
#[tokio::main]
async fn main() -> Result<(), BackendError> {
    // Parse configuration files
    let config = Configuration::parse()?;

    // Build tracing subscriber
    let tracing_subscriber = telemetry::get_tracing_subscriber(
        "authentication".into(),
        std::io::stdout,
        config.application.runtime_environment,
        config.application.log_level,
    );

    telemetry::init_tracing(tracing_subscriber)?;

    let database = database::init_pool(&config.database).await?;

    let tonic_server = startup::TonicServer::build(config, database).await?;
    let _ = tonic_server.run().await;

    Ok(())
}
