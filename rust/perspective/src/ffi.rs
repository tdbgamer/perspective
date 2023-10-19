use cxx::let_cxx_string;
use cxx::{SharedPtr, UniquePtr};

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
        include!("perspective/cpp/ffi.h");
        include!("perspective/cpp/types.h");

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

pub struct Column {
    column: SharedPtr<ffi_internal::Column>,
}

impl Column {
    pub fn get_dtype(&self) -> DType {
        ffi_internal::get_col_dtype(&self.column)
    }
}

pub struct Table {
    table: SharedPtr<ffi_internal::Table>,
}

impl Table {
    pub fn new() -> Self {
        let column_names = vec!["a".to_owned(), "psp_pkey".to_owned()];
        let data_types = vec![DType::DTYPE_INT64, DType::DTYPE_INT64];
        let limit = 100;
        let index = "a".to_string();
        let table = ffi_internal::mk_table(column_names, data_types, limit, index);
        Table { table }
    }

    pub fn get_column(&self, name: &str) -> Column {
        let gnode = self.table.get_gnode();
        let data_table = gnode.get_table_sptr();
        let_cxx_string!(n = name);
        let col = data_table.get_column(&n);
        Column { column: col }
    }

    pub fn size(&self) -> u32 {
        self.table.size()
    }
}
