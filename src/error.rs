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
pub enum Error {
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
	IO(#[from] std::io::Error)
}
