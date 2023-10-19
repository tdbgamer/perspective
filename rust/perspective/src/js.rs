use crate::ffi::{Pool, Table};
use wasm_bindgen::prelude::*;

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
pub fn make_table() -> *mut Table {
    let table = Box::new(Table::new());
    Box::into_raw(table)
}

#[wasm_bindgen]
pub fn get_col_dtype(table: *mut Table) -> String {
    let table = unsafe { &*table };
    // let col = table.get_column(&col);
    let col = table.get_column("a");
    let dtype = col.get_dtype();
    format!("{:?}", dtype)
}
