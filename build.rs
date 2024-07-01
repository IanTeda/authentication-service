use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<(dyn Error)>> {
    // -- Tonic Reflections
    // Setup tonic reflections so our clients don't need the *.proto files
    // let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // tonic_build::configure()
    //     .file_descriptor_set_path(out_dir.join("ledger_descriptor.bin"))
    //     .compile(&["proto/ledger.proto"], &["proto"])?;

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .build_transport(true)
        .compile_well_known_types(true)
        .file_descriptor_set_path(out_dir.join("ledger_descriptor.bin"))
        // .out_dir("src/proto/")
        // .include_file("mod.rs")
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            //&["proto/ServiceProtos/progressiveControllerSrv.proto", "proto/ServiceProtos/commonDataSrv.proto", "proto/Progressives/progressiveModels.proto", "proto/Common/common.proto"],
            &["proto/ledger.proto"],
            &["proto"])?;

    // compiling protos using path on build time 
    // tonic_build::compile_protos("proto/ledger.proto")?;

    Ok(())
}