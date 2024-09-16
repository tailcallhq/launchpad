use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("tailcall_descriptor.bin"))
        .compile(&["proto/tailcall.proto"], &["proto"])?;
    Ok(())
}
