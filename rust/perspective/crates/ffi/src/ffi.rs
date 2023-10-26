use cxx::let_cxx_string;
use cxx::{SharedPtr, UniquePtr};
use wasm_bindgen::prelude::*;

#[cxx::bridge(namespace = ffi)]
mod ffi_internal {
    #[derive(Debug)]
    enum DType {
        DTYPE_NONE,
        DTYPE_INT64,
        DTYPE_INT32,
        DTYPE_INT16,
        DTYPE_INT8,
        DTYPE_UINT64,
        DTYPE_UINT32,
        DTYPE_UINT16,
        DTYPE_UINT8,
        DTYPE_FLOAT64,
        DTYPE_FLOAT32,
        DTYPE_BOOL,
        DTYPE_TIME,
        DTYPE_DATE,
        DTYPE_ENUM,
        DTYPE_OID,
        DTYPE_OBJECT,
        DTYPE_F64PAIR,
        DTYPE_USER_FIXED,
        DTYPE_STR,
        DTYPE_USER_VLEN,
        DTYPE_LAST_VLEN,
        DTYPE_LAST,
    }

    unsafe extern "C++" {
        include!("perspective-ffi/cpp/ffi.h");
        include!("perspective-ffi/cpp/types.h");

        type Pool;
        type Table;
        type GNode;
        type DataTable;
        type Column;

        pub fn size(self: &Table) -> u32;
        pub fn get_gnode(self: &Table) -> SharedPtr<GNode>;

        pub fn get_table_sptr(self: &GNode) -> SharedPtr<DataTable>;

        pub fn get_column(self: &DataTable, name: &CxxString) -> SharedPtr<Column>;

        pub fn get_col_dtype(col: &Column) -> DType;
        pub fn get_col_nth_u32(col: &Column, idx: u32) -> u32;
        pub fn get_col_nth_u64(col: &Column, idx: u32) -> u64;
        pub fn get_col_nth_i32(col: &Column, idx: u32) -> i32;
        pub fn get_col_nth_i64(col: &Column, idx: u32) -> i64;
        pub fn pretty_print(table: &Table, num_rows: u32) -> String;

        pub fn size(self: &Column) -> u32;

        pub fn mk_pool() -> UniquePtr<Pool>;

        pub fn mk_table(
            column_names: Vec<String>,
            data_types: Vec<DType>,
            limit: u32,
            index: String,
        ) -> SharedPtr<Table>;

    }
}
pub use ffi_internal::mk_pool;
pub use ffi_internal::mk_table;
pub use ffi_internal::DType;

// use crate::cpp_common::DType;

pub struct Pool {
    pool: UniquePtr<ffi_internal::Pool>,
}

impl Pool {
    pub fn new() -> Self {
        Pool {
            pool: ffi_internal::mk_pool(),
        }
    }
}

#[wasm_bindgen]
pub struct Column {
    column: SharedPtr<ffi_internal::Column>,
}

impl Column {
    pub fn get_dtype(&self) -> DType {
        ffi_internal::get_col_dtype(&self.column)
    }

    pub fn get_u32(&self, idx: u32) -> u32 {
        ffi_internal::get_col_nth_u32(&self.column, idx)
    }

    pub fn get_u64(&self, idx: u32) -> u64 {
        ffi_internal::get_col_nth_u64(&self.column, idx)
    }
}

#[wasm_bindgen]
impl Column {
    #[wasm_bindgen(js_name = "getDType")]
    pub fn get_dtype_string(&self) -> String {
        format!("{:?}", self.get_dtype())
    }

    #[wasm_bindgen(js_name = "getU32")]
    pub fn get_u32_js(&self, idx: u32) -> u32 {
        self.get_u32(idx)
    }

    #[wasm_bindgen(js_name = "getU64")]
    pub fn get_u64_js(&self, idx: u32) -> u64 {
        self.get_u64(idx)
    }

    #[wasm_bindgen(js_name = "getI32")]
    pub fn get_i32_js(&self, idx: u32) -> i32 {
        ffi_internal::get_col_nth_i32(&self.column, idx)
    }

    #[wasm_bindgen(js_name = "getI64")]
    pub fn get_i64_js(&self, idx: u32) -> i64 {
        ffi_internal::get_col_nth_i64(&self.column, idx)
    }

    #[wasm_bindgen(js_name = "size")]
    pub fn size(&self) -> u32 {
        self.column.size()
    }
}

#[wasm_bindgen]
pub struct Table {
    table: SharedPtr<ffi_internal::Table>,
}

impl Table {}

#[wasm_bindgen]
impl Table {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let column_names = vec!["a".to_owned()];
        let data_types = vec![DType::DTYPE_INT64];
        let limit = 100;
        let index = "a".to_string();
        let table = ffi_internal::mk_table(column_names, data_types, limit, index);
        Table { table }
    }
    #[wasm_bindgen(js_name = "size")]
    pub fn size(&self) -> u32 {
        self.table.size()
    }

    #[wasm_bindgen(js_name = "getColumnDtype")]
    pub fn get_col_dtype(&self, col: String) -> String {
        let col = self.get_column(&col);
        let dtype = col.get_dtype();
        format!("{:?}", dtype)
    }

    #[wasm_bindgen(js_name = "getColumn")]
    pub fn get_column(&self, name: &str) -> Column {
        let gnode = self.table.get_gnode();
        let data_table = gnode.get_table_sptr();
        let_cxx_string!(n = name);
        let col = data_table.get_column(&n);
        Column { column: col }
    }

    #[wasm_bindgen(js_name = "prettyPrint")]
    pub fn pretty_print(&self, num_rows: u32) -> String {
        ffi_internal::pretty_print(&self.table, num_rows)
    }
}
