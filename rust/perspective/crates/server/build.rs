use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["../../protos/perspective.proto"], &["../../protos"])?;
    println!("cargo:rerun-if-changed=../../protos/perspective.proto");
    Ok(())
}
