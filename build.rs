//-- ./build.rs

//! # Build Script for Authentication Service
//!
//! This build script uses `tonic_build` to compile Protocol Buffer (`.proto`) files for gRPC service definitions.
//! It generates Rust client and server code, enables transport features, and outputs a file descriptor set for reflection or dynamic use into the OUT_DIR (project build target folder).
//! The OUT_DIR is typically something like `target/debug/build/authentication-<hash>/out` or `target/release/build/authentication-<hash>/out`. This directory is used by Cargo to store build artifacts, including generated code.
//!
//! ## Features
//! - Watches for changes in proto files and this build script to trigger recompilation.
//! - Supports proto3 optional fields via experimental protoc argument.
//! - Outputs generated code and descriptor set to the Cargo OUT_DIR.
//!
//! ## Proto Files
//! - authentication.proto
//! - common.proto
//! - sessions.proto
//! - users.proto
//! - utilities.proto
//!
//! ## Usage
//! This script is run automatically by Cargo during build. No manual invocation is required.
//!
//! ## References
//! - https://github.com/hyperium/tonic
//! - https://github.com/hyperium/tonic/blob/master/examples/build.rs

use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the cargo OUT_DIR environment variable, which is where the generated code will be placed
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // Ensure build script reruns if proto files or build.rs change
    println!("cargo:rerun-if-changed=proto/authentication/");
    println!("cargo:rerun-if-changed=build.rs");

    // Configure tonic_build to compile the proto files and generate Rust code
    tonic_build::configure()
        // Enable the `tonic` feature to generate client code
        .build_client(true)
        // Enable the `tonic` feature to generate server code
        .build_server(true)
        // Enable the `tonic` feature to generate transport code
        .build_transport(true)
        .compile_well_known_types(true)
        .file_descriptor_set_path(out_dir.join("authentication_descriptor.bin"))
        // Include experimental proto3 optional support
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(
            // Proto files to compile
            &[
                "proto/authentication/authentication.proto",
                "proto/authentication/common.proto",
                "proto/authentication/sessions.proto",
                "proto/authentication/users.proto",
                "proto/authentication/utilities.proto",
            ],
            // Proto root directory for imports, this is relevant for the proto file imports.
            &["proto/authentication"],
        )?;

    Ok(())
}
