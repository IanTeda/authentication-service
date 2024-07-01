//-- ./src/rpc/reflections.rs

//! Return a result containing a RPC reflections server

// #![allow(unused)] // For beginning only.

use tonic_reflection::server::{ServerReflection, ServerReflectionServer};
use super::proto;

use crate::prelude::*;

pub fn get_reflection() -> Result<ServerReflectionServer<impl ServerReflection>, BackendError> {
    let reflections_server = 
        tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(proto::DESCRIPTOR_SET)
            .build()?;
    
    Ok(reflections_server)
}
