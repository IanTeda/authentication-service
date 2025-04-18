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

use crate::AuthenticationError;

/// # Get Cookies
///
/// Retrieve all the cookies in the request metadata, returning a cookie jar
#[tracing::instrument(name = "Collect cookies into a cookie jar: ")]
pub fn get_cookie_jar(
    metadata: &tonic::metadata::MetadataMap,
) -> Result<CookieJar, AuthenticationError> {
    // Collect all cookie headers from the request metadata into a cookies vector
    let cookies = metadata.get_all("cookie").into_iter().collect::<Vec<_>>();
    tracing::debug!("Cookies collected from the header: {cookies:#?}");

    // Create a cookie jar for storing all the header cookies in
    let mut cookie_jar = CookieJar::new();

    // Iterate through cookie headers, and add to to cookie jar
    for cookie in cookies {
        // Convert Cookie Ascii into a string for splitting
        let cookie = match cookie.to_str() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("{:?}: Error converting cookie Ascii to string! Skipping adding it to the  jar: {:?}", e, cookie);
                continue;
            }
        };

        // Split the cookie string into sub-cookies
        let sub_cookies: Vec<&str> = cookie.split(";").collect();

        // Inetrate over sub-cookies, adding them to the cookie jar
        for sub_cookie in sub_cookies {
            // Parse sub cookie into a Cookie
            let sub_cookie = match Cookie::parse(sub_cookie) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("{:?}: Error parsing string to cookie!  Skipping adding it to the  jar: {:?}", e, sub_cookie);
                    continue;
                }
            };

            // Add the cookie to the cookie jar
            cookie_jar.add_original(sub_cookie.into_owned());
        }
    }

    tracing::info!("Cookie jar collected: {cookie_jar:#?}");

    Ok(cookie_jar)
}
