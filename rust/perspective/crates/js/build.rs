fn main() {
    prost_build::compile_protos(&["../../protos/perspective.proto"], &["../../protos/"]).unwrap();
}
