fn main() {
    prost_build::compile_protos(&["../../protos/perspective.proto"], &["../../protos/"]).unwrap();
    println!("cargo:rerun-if-changed=../../protos/perspective.proto");
}
