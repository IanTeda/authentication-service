//-- ./src/errors.rs

#![allow(unused)] // For beginning only.

//! Main Crate Error
//! # References
//! 
//! * [Rust Error Handling - Best Practices](https://www.youtube.com/watch?v=j-VQCYP7wyw)
//! * [jeremychone-channel/rust-base](https://github.com/jeremychone-channel/rust-base)
//! * [derive(Error)](https://github.com/dtolnay/thiserror)
//! * [How to Handle Errors in Rust: A Comprehensive Guide](https://dev.to/nathan20/how-to-handle-errors-in-rust-a-comprehensive-guide-1cco)
//! * [Rust Error Types Explained: Building Robust Error Handling](https://marketsplash.com/rust-error-types/)

/// Static errors types
#[derive(thiserror::Error, Debug)]
pub enum BackendError {
	//-- Generic Errors
	/// For starter, to remove as code matures.
	#[error("Generic error: {0}")]
	Generic(String),
	/// For starter, to remove as code matures.
	#[error("Static error: {0}")]
	Static(&'static str),

	//-- Module errors
	// #[error("{name:?} is not a valid Thing name.")]
	// ConfigUnsupportedEnvironmentError {
	// 	environment: String,
	// },

	//-- External errors
	/// Derive IO errors
	#[error(transparent)]
	IO(#[from] std::io::Error),
	// Config errors
	#[error(transparent)]
    Config(#[from] config::ConfigError),
	// Tonic Reflections errors
	#[error(transparent)]
    TonicReflection(#[from] tonic_reflection::server::Error),
	// Tonic transport errors
	#[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),
	// Standard network address error
	#[error(transparent)]
    AddressParse(#[from] std::net::AddrParseError),
	// Environmental parse error
	// #[error(transparent)]
	// EnvironmentParse(#[from] std::env::VarError),
	#[error(transparent)]
    LogError(#[from] tracing_log::log::SetLoggerError),
	#[error(transparent)]
    TracingError(#[from] tracing::dispatcher::SetGlobalDefaultError),
}
