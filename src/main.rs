//-- ./src/main.rs

// #![allow(unused)] // For beginning only.

mod configuration;
mod error;
mod prelude;
mod rpc;
mod services;
mod startup;
mod telemetry;
mod utils;

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

	let tonic = startup::TonicServer::build(settings).await?;
	let _ = tonic.run().await;

	Ok(())
}
