// -- ./src/configuration.rs

// #![allow(unused)] // For development only

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
//! - [Example 2](https://github.com/stoically/web-service-rs-template/blob/main/src/config.rs)

use crate::prelude::*;

use secrecy::{ExposeSecret, SecretString};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use strum::Display;
use serde_with::{serde_as, DisplayFromStr};
use tracing_subscriber::filter as tracing;

/// Configuration for the API
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Configuration {
    /// Application configuration
    pub application: ApplicationConfiguration,

    /// Database configuration
    pub database: DatabaseConfiguration,
}

/// Configuration for running the API application
#[serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApplicationConfiguration {
    /// The host address the api should bind to
    pub ip_address: String,

    /// The port that the api should bind to
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,

    #[serde_as(as = "DisplayFromStr")]
    pub log_level: tracing::LevelFilter,

    /// Secret used to generate JWT keys
    pub token_secret: SecretString,

    /// How many minutes to keep the access token valid for.
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub access_token_duration_minutes: u64,

    /// How many minutes to keep the access token valid for.
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub refresh_token_duration_minutes: u64,
}

/// Configuration for connecting to the database server
#[derive(Debug,Clone, serde::Deserialize)]
pub struct DatabaseConfiguration {
    /// Database host address
    pub host: String,

    /// Database host port
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,

    /// Database username for login
    pub username: String,

    /// Database password for login
    pub password: SecretString,

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
        // Pgpass allows for storing postgres passwords int he users directory.
        // We are not going to use that.
        PgConnectOptions::new_without_pgpass()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
            .database(&self.database_name)
            .ssl_mode(ssl_mode)
    }
}

/// The possible runtime environment for our application.
#[derive(Clone, Debug, PartialEq, Copy, serde::Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Environment {
    Development,
    Testing,
    Production,
}

// TODO: why is this dead co
#[allow(dead_code)]
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

impl Configuration {
    /// Parse the application configuration, returning a `Configuration` result.
    pub fn parse() -> Result<Configuration, AuthenticationError> {
        // Get the directory that the binary is being run from
        let base_path = std::env::current_dir()
            .expect("Failed to determine the current directory");

        // Set the configuration directory for the app
        let configuration_directory = base_path.join("configuration");

        // Set the default config file path
        let default_config_file = configuration_directory.join("default.yaml");

        // Get the runtime environment the binary was started in
        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "development".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.");

        // Set the environment config file path
        let environment_config_file =
            configuration_directory.join(format!("{}.yaml", environment));

        // Build our configuration instance. Configuration files are added in
        // this order, with subsequent files overwriting previous configurations
        // if present.
        let settings_builder = config::Config::builder()
            .add_source(config::File::from(default_config_file))
            .add_source(config::File::from(environment_config_file))
            // Add in settings from environment variables (with a prefix of BACKEND
            // and '_' as separator). E.g. `BACKEND_APPLICATION_PORT=5001 would
            // set `settings.application.port`
            .add_source(
                config::Environment::with_prefix("AUTHENTICATION")
                    .prefix_separator("_")
                    .separator("_"),
            )
            .build()?;

        let configuration = settings_builder.try_deserialize::<Configuration>()?;

        println!(
            "\n----------- CONFIGURATION ----------- \n{:#?} \n-------------------------------------",
            configuration
        );

        // Convert the configuration values into Settings type
        Ok(configuration)
    }


}

impl ApplicationConfiguration {
    /// # Get Token Issuer
    /// 
    /// This function returns the issuer URL for the token. The issuer URL is
    /// used to validate the token issuer when parsing the token.
    ///
    /// # Returns
    ///
    /// * `Secret<String>` - The issuer URL for the token.
    pub fn get_issuer(&self) -> SecretString {
        let issuer = format!(
            "https://{}:{}",
            self.ip_address, self.port
        );
        SecretString::from(issuer)
    }

    /// # Get the Cookie Domain
    /// 
    /// Cookies are set to website domain.
    pub fn get_domain(&self) -> String {
        let domain = format!(
            "http://{}",
            self.ip_address
        );
        domain
    }

    /// # Get the Server Address
    ///
    /// This function returns the server address for the API. The server address
    /// is used to bind the server to the correct address and port.
    pub fn get_address(&self) -> String {
        format!("{}:{}", self.ip_address, self.port)
    }
}
