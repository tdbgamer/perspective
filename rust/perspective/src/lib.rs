pub mod api;
pub mod cpp_common;
pub mod ffi;
pub mod js;

// WASI reactor initialization function.
// This is because we're not running a `main` entrypoint that
// results in an exit code, we're initializing a library such that
// we can consume its functions from JavaScript.
#[no_mangle]
extern "C" fn _initialize() {
    println!("Hello, world!");
}
