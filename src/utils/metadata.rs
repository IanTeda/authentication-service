//-- ./src/utils/header.rs

// #![allow(unused)] // For beginning only.

//! # Metadata Utilities
//!
//! Utility functions for working with request and response http Metadata (headers)
//!
//! Modules include:
//!
//! - `get_cookies(header: &tonic::metadata::MetadataMap)` - returns a cookie jar of http cookies
//! 
//! ## TODO
//! 
//! - [ ] Write unit tests

use cookie::{Cookie, CookieJar};

use crate::BackendError;

/// # Get Cookies
///
/// Retrive all the cookies in the request metadata, returning a cookie jar
pub fn get_cookie_jar(metadata: &tonic::metadata::MetadataMap) -> Result<CookieJar, BackendError> {
    // Collect all cookies from the request metadata into a cookies vector
    let cookies = metadata.get_all("cookie").into_iter().collect::<Vec<_>>();
    tracing::debug!("Cookies vector: {cookies:#?}");

    // Create a cookie jar for storing all the header cookies in
    let mut cookie_jar = CookieJar::new();

    for cookie in cookies {
      // Convert the cookie to a string
      let cookie = cookie.to_str().map_err(|_| {
          BackendError::AuthenticationError(
              "Error converting cookie Ascii to string".to_string(),
          )
      })?;

      // Parse the cookie string into a cookie object
      let cookie = Cookie::parse(cookie).map_err(|_| {
        tracing::error!("Error parsing string to cookie");
        BackendError::AuthenticationError(
            "Error converting cookie Ascii to string".to_string(),
        )
      })?;

      // Add the cookie to the cookie jar
      cookie_jar.add_original(cookie.into_owned());
    }

    Ok(cookie_jar)
}
