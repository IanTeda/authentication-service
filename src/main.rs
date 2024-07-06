//-- ./src/main.rs

// #![allow(unused)] // For beginning only.

// For intellisense
mod configuration;
mod database;
mod domains;
mod error;
mod prelude;
mod middleware;
mod reflections;
mod router;
mod rpc;
mod services;
mod startup;
mod telemetry;
mod utilities;

use crate::prelude::*;

/// Binary entry point
#[tokio::main]
async fn main() -> Result<(), BackendError> {
	// Parse configuration files
	let settings = configuration::Settings::parse()?;

	// Build tracing subscriber
	let tracing_subscriber = telemetry::get_tracing_subscriber(
		"personal_ledger_server".into(),
		std::io::stdout,
		settings.application.runtime_environment,
		settings.application.log_level,
	);

	telemetry::init_tracing(tracing_subscriber)?;

	let database = database::init_pool(&settings.database).await?;

	let tonic_server = startup::TonicServer::build(settings, database).await?;
	let _ = tonic_server.run().await;

	Ok(())
}
