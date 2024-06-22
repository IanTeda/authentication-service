use std::error::Error;
use std::{env, path::PathBuf};

fn main() -> Result<(), Box<(dyn Error)>> {
    // -- Tonic Reflections
    // Setup tonic reflections so our clients don't need the *.proto files
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("ledger_descriptor.bin"))
        .compile(&["proto/ledger.proto"], &["proto"])?;

    // compiling protos using path on build time 
    tonic_build::compile_protos("proto/ledger.proto")?;

    Ok(())
}