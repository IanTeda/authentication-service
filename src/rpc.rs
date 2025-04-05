//-- ./src/rpc.rs

//! # RPC Protocol Buffers
//! 
//! This module contains the RPC protocol buffers for the authentication service.

#![allow(unused)] // For development only

use tonic_reflection::server::v1::{ ServerReflection, ServerReflectionServer};

use crate::prelude::*;

/// # RPC Protocol Buffers
/// 
/// This module contains the RPC protocol buffers for the authentication service.
/// The `proto` module contains the generated code from the Protobuf files.
/// The `spec_service` function returns a reflection server to allow reading the proto definition at runtime.
pub mod proto {
    // The string specified here must match the proto package name
    tonic::include_proto!("authentication");

    #[allow(dead_code)]
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("authentication_descriptor");
}

// spec_service returns reflection server to allow reading proto definition at runtime.
pub fn spec_service() -> Result<ServerReflectionServer<impl ServerReflection>, AuthenticationError> {
    // Create the reflection service
    // This service allows us to read the proto definition at runtime
    // and is used by gRPC-Web to generate the client code.
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    Ok(reflection_service)
}

