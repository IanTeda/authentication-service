use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<(dyn Error)>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .build_transport(true)
        .compile_well_known_types(true)
        .out_dir("src/rpc/")
        .include_file("mod.rs")
        .file_descriptor_set_path(out_dir.join("ledger_descriptor.bin"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["proto/ledger.proto"], &["proto"])?;

    Ok(())
}
