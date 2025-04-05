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
async fn main() -> Result<(), AuthenticationError> {
    // Parse configuration files
    let config = Configuration::parse()?;

    // Start tracing
    let log_level = config.application.log_level;
    let _telemetry = telemetry::init(log_level)?;

    let database = database::init_pool(&config.database).await?;

    let tonic_server = startup::TonicServer::build(config, database).await?;
    let _tonic_server = tonic_server.run().await;

    Ok(())
}
