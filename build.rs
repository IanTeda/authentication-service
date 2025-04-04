use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

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
            // Proto root directory for imports
            &["proto/authentication"],
        )?;

    Ok(())
}
