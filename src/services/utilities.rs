//-- ./src/rpc/utilities.rs

//! Return a result containing a RPC Utilities server

// #![allow(unused)] // For beginning only.

use std::sync::Arc;

use crate::configuration::Configuration;
use tonic::{Request, Response, Status};

use crate::rpc::ledger::utilities_server::Utilities;
use crate::rpc::ledger::{Empty, PingResponse};

// #[derive(Debug, Default)]
pub struct UtilitiesService {
    #[allow(dead_code)]
    config: Arc<Configuration>,
}

impl UtilitiesService {
    pub fn new(config: Arc<Configuration>) -> Self {
        Self { config }
    }
}

#[tonic::async_trait]
impl Utilities for UtilitiesService {
    #[tracing::instrument(
		name = "Ping endpoint",
		skip(self),
	)]
    async fn ping(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<PingResponse>, Status> {
        let response_message = PingResponse {
            message: "Pong...".to_string(),
        };

        // Send back our ping response.
        Ok(Response::new(response_message))
    }
}
