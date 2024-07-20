use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

   let _tonic_build = tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .build_transport(true)
        .compile_well_known_types(true)
        .file_descriptor_set_path(out_dir.join("authentication_descriptor.bin"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &[
                "./proto/common.proto",
                "./proto/authentication.proto",
                "./proto/refresh_tokens.proto",
                "./proto/users.proto",
                "./proto/utilities.proto",
            ],
            &["./proto"],
        )?;

    Ok(())
}
