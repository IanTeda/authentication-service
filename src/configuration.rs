// -- ./src/configuration.rs

//! Application configuration settings
//!
//! # Application Configuration Crate
//!
//! Get API configuration from the `./configuration/base.yaml` file and
//! overwrite with runtime environment configuration `./config/production.yaml`
//! and environmental runtime variables.
//!
//! # References
//!
//! * [config.rs Repository](https://github.com/mehcode/config-rs)
//! * [Configuration management in Rust web services](https://blog.logrocket.com/configuration-management-in-rust-web-services/)

// #![allow(unused)] // For development only

use crate::prelude::*;

use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::path::PathBuf;
use strum::{AsRefStr, Display};

/// Directory from binary base folder to look in for configuration files
const CONFIGURATION_DIRECTORY_PREFIX: &str = "configuration/";
/// If the configuration files do not set this default is used.
const DEFAULT_RUNTIME_ENVIRONMENT: &str = "development";
/// If the configuration files do not set this default is used.
const DEFAULT_LOG_LEVEL: &str = "info";
/// If the configuration files do not set this default is used.
const DEFAULT_QUERY_OFFSET: i64 = 0;
/// If the configuration files do not set this default is used.
const DEFAULT_QUERY_LIMIT: i64 = 10;

/// Configuration for the API
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Configuration {
	pub database: DatabaseConfiguration,
	pub application: ApplicationConfiguration,
}

/// Define log levels the system will recognise
#[derive(serde::Deserialize, Debug, Clone, AsRefStr, Display, Copy)]
pub enum LogLevels {
	Error,
	Warn,
	Info,
	Debug,
	Trace,
}

/// Configuration for running the API application
#[derive(serde::Deserialize, Clone, Debug)]
pub struct ApplicationConfiguration {
	// The host address the api should bind to
	pub ip_address: String,
	/// The port that the api should bind to
	#[serde(deserialize_with = "deserialize_number_from_string")]
	pub port: u16,
	/// Application log level has a default set in builder
	pub log_level: LogLevels,
	/// Application runtime environment is set to default in the builder
	pub runtime_environment: Environment,
	// Secret used to generate JWT keys
	pub jwt_secret: String,
	/// Default application settings
	#[allow(dead_code)]
	pub default: DefaultApplicationSettings,
}

/// Default application settings
#[derive(serde::Deserialize, Clone, Debug)]
pub struct DefaultApplicationSettings {
	// Default sql query offset
	#[allow(dead_code)]
	pub query_offset: i64,
	// Default sql query limit
	#[allow(dead_code)]
	pub query_limit: i64,
}

/// Configuration for connecting to the database server
#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseConfiguration {
	/// Database host address
	pub host: String,
	/// Database host port
	#[serde(deserialize_with = "deserialize_number_from_string")]
	pub port: u16,
	/// Database username for login
	pub username: String,
	/// Database password for login
	pub password: Secret<String>,
	/// Database name to use
	pub database_name: String,
	/// Should ssl be used to connect to the database
	pub require_ssl: bool,
}

impl DatabaseConfiguration {
	/// Build database connection
	pub fn connection(&self) -> PgConnectOptions {
		let ssl_mode = if self.require_ssl {
			PgSslMode::Require
		} else {
			PgSslMode::Prefer
		};
		PgConnectOptions::new()
			.host(&self.host)
			.port(self.port)
			.username(&self.username)
			.password(self.password.expose_secret())
			.database(&self.database_name)
			.ssl_mode(ssl_mode)
	}
}

/// The possible runtime environment for our application.
#[derive(Clone, Debug, serde::Deserialize, PartialEq, Copy, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Environment {
	Development,
	Testing,
	Production,
}

impl Environment {
	pub fn as_str(&self) -> &'static str {
		match self {
			Environment::Development => "development",
			Environment::Testing => "testing",
			Environment::Production => "production",
		}
	}
}

impl TryFrom<String> for Environment {
	type Error = String;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "testing" => Ok(Self::Testing),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `development`, `testing` or `production`.",
                other
            )),
        }
	}
}

/// Returns the runtime environment enum used to start the application
///
/// This function parse the runtime environmental variables for "APP_ENVIRONMENT".
/// If the variable is not set, then default to development
pub fn get_runtime_environment() -> Result<Environment, BackendError> {
	let environment = std::env::var("APP_ENVIRONMENT")
		.unwrap_or_else(|_| "development".into())
		.try_into()
		.expect("Failed to parse APP_ENVIRONMENT.");
	Ok(environment)
}

impl Configuration {
	/// Parse the application configuration from yaml files, returning a
	/// `Configuration` result.
	pub fn parse() -> Result<Configuration, BackendError> {
		// Define the configuration directory within the base application directory
		let base_dir_path: PathBuf = std::env::current_dir()?.join(CONFIGURATION_DIRECTORY_PREFIX);

		let environment_filename = format!("{}.yaml", get_runtime_environment()?.as_str());

		// Build our configuration instance. Configuration files are added in
		// this order, with subsequent files overwriting previous configurations
		// if present.
		let settings_builder = config::Config::builder()
			.set_default(
				"application.runtime_environment",
				DEFAULT_RUNTIME_ENVIRONMENT,
			)?
			.set_default("application.log_level", DEFAULT_LOG_LEVEL)?
			.set_default("application.default.query_offset", DEFAULT_QUERY_OFFSET)?
			.set_default("application.default.query_limit", DEFAULT_QUERY_LIMIT)?
			.add_source(config::File::from(base_dir_path.join("default.yaml")))
			.add_source(config::File::from(base_dir_path.join(environment_filename)))
			// -- Environmental variables
			// Add in settings from environment variables (with a prefix of BACKEND
			// and '__' as separator). E.g. `BACKEND__APPLICATION_PORT=5001 would
			// set `settings.application.port`
			.add_source(
				config::Environment::with_prefix("BACKEND")
					.prefix_separator("_")
					.separator("__"),
			)
			.build()?;

		let configuration = settings_builder.try_deserialize::<Configuration>()?;

		// println!(
		//     "\n----------- CONFIGURATION ----------- \n{:?} \n-------------------------------------",
		//     configuration
		// );

		// Convert the configuration values into Settings type
		Ok(configuration)
	}
}
