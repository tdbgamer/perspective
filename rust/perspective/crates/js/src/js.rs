use perspective_ffi::{Pool, Table};
use wasm_bindgen::prelude::*;

// WASI reactor initialization function.
// This is because we're not running a `main` entrypoint that
// results in an exit code, we're initializing a library such that
// we can consume its functions from JavaScript.
#[no_mangle]
extern "C" fn _initialize() {
    println!("Hello, world!");
}

#[wasm_bindgen]
pub fn make_pool() -> *mut Pool {
    Box::into_raw(Box::new(Pool::new()))
}

#[wasm_bindgen]
pub fn free_pool(ptr: *mut Pool) {
    unsafe {
        drop(Box::from_raw(ptr));
    }
}

#[wasm_bindgen]
pub fn make_table() -> Table {
    Table::new()
}

#[wasm_bindgen]
pub fn get_col_dtype(table: *mut Table, col: String) -> String {
    let table = unsafe { &*table };
    let col = table.get_column(&col);
    let dtype = col.get_dtype();
    format!("{:?}", dtype)
}
