pub mod cpp_common;
pub mod ffi;
pub mod js;

#[no_mangle]
extern "C" fn _initialize() {
    println!("Hello, world!");
}
