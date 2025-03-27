//-- ./src/middleware/token_interceptor.rs

#![allow(unused)] // For beginning only.

use secrecy::Secret;
use crate::{domain, middleware::{access_token, refresh_token}, prelude::*};


pub struct TokenInterceptor {
    pub(crate) token_secret: Secret<String>,
}

impl tonic::service::Interceptor for TokenInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
      tracing::debug!("Token Interceptor called");

      let request_metadata = request.metadata();
      tracing::debug!("Request Metadata: {:?}", request_metadata);

      let access_token = request_metadata.get("access_token");
      

      let refresh_token = request_metadata.get("refresh_token");
      

      Ok(request)
    }
  }