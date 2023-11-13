use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::Arc;

use arrow_schema::{DataType, TimeUnit};
use cxx::let_cxx_string;
use cxx::{SharedPtr, UniquePtr};
use std::pin::Pin;
use wasm_bindgen::{prelude::*, JsObject};

#[cxx::bridge(namespace = ffi)]
mod ffi_internal {

    #[derive(Debug, Eq, PartialEq)]
    #[repr(u8)]
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
        type Schema;

        pub fn size(self: &Table) -> usize;
        pub fn get_gnode(self: &Table) -> SharedPtr<GNode>;
        pub fn make_table_port(table: &Table) -> usize;

        pub fn get_table_schema(table: &Table) -> UniquePtr<Schema>;

        pub fn get_schema_uptr(self: &DataTable) -> UniquePtr<Schema>;
        pub fn table_extend(table: UniquePtr<DataTable>, size: usize) -> UniquePtr<DataTable>;

        pub fn get_schema_columns(schema: &Schema) -> Vec<String>;
        pub fn get_schema_types(schema: &Schema) -> Vec<DType>;

        pub fn get_table_sptr(self: &GNode) -> SharedPtr<DataTable>;
        pub fn process_gnode(gnode: &GNode, idx: usize) -> bool;

        pub fn get_column(self: &DataTable, name: &CxxString) -> SharedPtr<Column>;

        pub fn get_col_dtype(col: &Column) -> DType;
        pub fn get_col_nth_u32(col: &Column, idx: usize) -> u32;
        pub fn get_col_nth_u64(col: &Column, idx: usize) -> u64;
        pub fn get_col_nth_i32(col: &Column, idx: usize) -> i32;
        pub fn get_col_nth_i64(col: &Column, idx: usize) -> i64;

        pub unsafe fn fill_column_u32(
            col: SharedPtr<Column>,
            ptr: *const u32,
            start: usize,
            len: usize,
        );

        pub unsafe fn fill_column_u64(
            col: SharedPtr<Column>,
            ptr: *const u64,
            start: usize,
            len: usize,
        );

        pub unsafe fn fill_column_i32(
            col: SharedPtr<Column>,
            ptr: *const i32,
            start: usize,
            len: usize,
        );

        pub unsafe fn fill_column_i64(
            col: SharedPtr<Column>,
            ptr: *const i64,
            start: usize,
            len: usize,
        );

        pub unsafe fn fill_column_f32(
            col: SharedPtr<Column>,
            ptr: *const f32,
            start: usize,
            len: usize,
        );

        pub unsafe fn fill_column_f64(
            col: SharedPtr<Column>,
            ptr: *const f64,
            start: usize,
            len: usize,
        );

        pub unsafe fn fill_column_date(
            col: SharedPtr<Column>,
            ptr: *const i32,
            start: usize,
            len: usize,
        );

        pub unsafe fn fill_column_time(
            col: SharedPtr<Column>,
            ptr: *const i64,
            start: usize,
            len: usize,
        );

        pub unsafe fn fill_column_dict(
            col: SharedPtr<Column>,
            dict: *const c_char,
            offsets: &[i32],
            ptr: *const i32,
            start: usize,
            len: usize,
        );

        pub fn pretty_print(table: &Table, num_rows: usize) -> String;

        pub fn size(self: &Column) -> usize;

        pub fn mk_pool() -> UniquePtr<Pool>;

        pub fn mk_table_from_data_table(
            data_table: UniquePtr<DataTable>,
            index: &CxxString,
        ) -> SharedPtr<Table>;

        pub fn mk_table(
            column_names: Vec<String>,
            data_types: Vec<DType>,
            limit: u32,
            index: String,
        ) -> SharedPtr<Table>;

        pub fn mk_schema(column_names: Vec<String>, data_types: Vec<DType>) -> UniquePtr<Schema>;
        pub fn mk_data_table(schema: &Schema, capacity: usize) -> UniquePtr<DataTable>;
    }
}
pub use ffi_internal::mk_pool;
pub use ffi_internal::mk_table;
pub use ffi_internal::DType;

// use crate::cpp_common::DType;

pub struct DataTable {
    data_table: UniquePtr<ffi_internal::DataTable>,
}

impl DataTable {
    pub fn new(schema: Schema, capacity: usize) -> Self {
        DataTable {
            data_table: ffi_internal::mk_data_table(&schema.schema, capacity),
        }
    }

    pub fn get_schema(&self) -> Schema {
        let schema = self.data_table.get_schema_uptr();
        Schema { schema }
    }

    pub fn get_columns(&self) -> Vec<String> {
        ffi_internal::get_schema_columns(&self.data_table.get_schema_uptr())
    }

    pub fn get_column(&self, name: &str) -> Column {
        let_cxx_string!(n = name);
        let col = self.data_table.get_column(&n);
        Column { column: col }
    }
}

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

    pub fn get_u32(&self, idx: usize) -> u32 {
        ffi_internal::get_col_nth_u32(&self.column, idx)
    }

    pub fn get_u64(&self, idx: usize) -> u64 {
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
    pub fn get_u32_js(&self, idx: usize) -> u32 {
        self.get_u32(idx)
    }

    #[wasm_bindgen(js_name = "getU64")]
    pub fn get_u64_js(&self, idx: usize) -> u64 {
        self.get_u64(idx)
    }

    #[wasm_bindgen(js_name = "getI32")]
    pub fn get_i32_js(&self, idx: usize) -> i32 {
        ffi_internal::get_col_nth_i32(&self.column, idx)
    }

    #[wasm_bindgen(js_name = "getI64")]
    pub fn get_i64_js(&self, idx: usize) -> i64 {
        ffi_internal::get_col_nth_i64(&self.column, idx)
    }

    #[wasm_bindgen(js_name = "size")]
    pub fn size(&self) -> usize {
        self.column.size()
    }
}

#[wasm_bindgen]
pub struct Schema {
    schema: UniquePtr<ffi_internal::Schema>,
}

impl Schema {
    pub fn new(columns: HashMap<String, DType>) -> Self {
        let (keys, values): (Vec<String>, Vec<DType>) = columns.into_iter().unzip();
        Schema {
            schema: ffi_internal::mk_schema(keys, values),
        }
    }

    pub fn columns(&self) -> Vec<String> {
        ffi_internal::get_schema_columns(&self.schema)
    }

    pub fn types(&self) -> Vec<DType> {
        ffi_internal::get_schema_types(&self.schema)
    }
}

impl TryFrom<Arc<arrow_schema::Schema>> for Schema {
    type Error = perspective_api::Error;

    fn try_from(value: Arc<arrow_schema::Schema>) -> Result<Self, Self::Error> {
        let mut columns = HashMap::new();
        for field in value.fields().iter() {
            let dtype = match field.data_type() {
                arrow_schema::DataType::Int32 => DType::DTYPE_INT32,
                arrow_schema::DataType::Int64 => DType::DTYPE_INT64,
                arrow_schema::DataType::UInt32 => DType::DTYPE_UINT32,
                arrow_schema::DataType::UInt64 => DType::DTYPE_UINT64,
                arrow_schema::DataType::Float32 => DType::DTYPE_FLOAT32,
                arrow_schema::DataType::Float64 => DType::DTYPE_FLOAT64,
                arrow_schema::DataType::Date32 => DType::DTYPE_DATE,
                arrow_schema::DataType::Date64 => DType::DTYPE_TIME,
                arrow_schema::DataType::Dictionary(k, v) => match (k.as_ref(), v.as_ref()) {
                    (&arrow_schema::DataType::Int32, &arrow_schema::DataType::Utf8) => {
                        DType::DTYPE_STR
                    }
                    _ => {
                        return Err(perspective_api::Error::UnsupportedArrowSchema(format!(
                            "{:?}",
                            field.data_type()
                        )))
                    }
                },
                _ => {
                    return Err(perspective_api::Error::UnsupportedArrowSchema(format!(
                        "{:?}",
                        field.data_type()
                    )))
                }
            };
            columns.insert(field.name().to_owned(), dtype);
        }
        Ok(Schema::new(columns))
    }
}

#[wasm_bindgen]
pub struct Table {
    table: SharedPtr<ffi_internal::Table>,
}
// TODO: Figure out why this is necessary. No matter what I do,
//       it seems to choke on Sending C++ types since they wrap a void*
unsafe impl Send for Table {}
unsafe impl Sync for Table {}

#[wasm_bindgen]
impl Table {
    // TODO: Flesh this out more.
    pub fn from_csv(csv: String) -> Table {
        todo!()
    }
    pub fn from_arrow(bytes: Vec<u8>) -> Table {
        todo!()
    }
    pub fn from_json(json: String) -> Table {
        todo!()
    }
    // END TODO

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
    pub fn size(&self) -> usize {
        self.table.size()
    }

    #[wasm_bindgen(js_name = "process")]
    pub fn process(&self) {
        ffi_internal::process_gnode(&self.table.get_gnode(), 0);
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
    pub fn pretty_print(&self, num_rows: usize) -> String {
        ffi_internal::pretty_print(&self.table, num_rows)
    }

    #[wasm_bindgen(js_name = "makePort")]
    pub fn make_port(&self) -> usize {
        ffi_internal::make_table_port(&self.table)
    }
}

impl Table {
    pub fn schema(&self) -> Schema {
        let schema = ffi_internal::get_table_schema(&self.table);
        Schema { schema }
    }

    pub fn columns(&self) -> Vec<String> {
        self.schema().columns()
    }
}

pub fn read_arrow(bytes: &[u8]) -> perspective_api::Result<Table> {
    let cursor = std::io::Cursor::new(bytes);
    let reader = arrow_ipc::reader::StreamReader::try_new(cursor, None).unwrap();
    let reader_schema = reader.schema();
    let schema = Schema::try_from(reader.schema()).unwrap();
    let mut data_table = DataTable::new(schema, 0);
    let mut start_at = 0;
    let columns = reader_schema
        .all_fields()
        .iter()
        .map(|x| x.name())
        .collect::<Vec<_>>();
    for batch_res in reader {
        let batch = batch_res.unwrap();
        data_table = DataTable {
            data_table: ffi_internal::table_extend(data_table.data_table, batch.num_rows()),
        };
        let col_with_names = columns
            .iter()
            .map(|&n| (n, batch.column_by_name(n).unwrap()))
            .collect::<Vec<_>>();
        for (n, col) in col_with_names {
            let tpe: &DataType = col.data_type();
            let pcol = data_table.get_column(&n);
            let mk_err = || {
                Err(perspective_api::Error::UnsupportedArrowSchema(format!(
                    "Unsupported arrow schema: {:?}",
                    tpe
                )))
            };
            match tpe {
                DataType::Null => return mk_err(),
                DataType::Boolean => return mk_err(),
                DataType::Int8 => return mk_err(),
                DataType::Int16 => return mk_err(),
                DataType::Int32 => {
                    let int32_data = col
                        .as_any()
                        .downcast_ref::<arrow_array::Int32Array>()
                        .unwrap();
                    let ptr = int32_data.values().as_ptr();
                    unsafe {
                        ffi_internal::fill_column_i32(pcol.column.clone(), ptr, start_at, col.len())
                    };
                }
                DataType::Int64 => {
                    let int64_data = col
                        .as_any()
                        .downcast_ref::<arrow_array::Int64Array>()
                        .unwrap();
                    let ptr = int64_data.values().as_ptr();
                    unsafe {
                        ffi_internal::fill_column_i64(pcol.column.clone(), ptr, start_at, col.len())
                    };
                }
                DataType::UInt8 => return mk_err(),
                DataType::UInt16 => return mk_err(),
                DataType::UInt32 => {
                    let uint32_data = col
                        .as_any()
                        .downcast_ref::<arrow_array::UInt32Array>()
                        .unwrap();
                    let ptr = uint32_data.values().as_ptr();
                    unsafe {
                        ffi_internal::fill_column_u32(pcol.column.clone(), ptr, start_at, col.len())
                    };
                }
                DataType::UInt64 => {
                    let uint64_data = col
                        .as_any()
                        .downcast_ref::<arrow_array::UInt64Array>()
                        .unwrap();
                    let ptr = uint64_data.values().as_ptr();
                    unsafe {
                        ffi_internal::fill_column_u64(pcol.column.clone(), ptr, start_at, col.len())
                    };
                }
                DataType::Float16 => return mk_err(),
                DataType::Float32 => {
                    let float32_data = col
                        .as_any()
                        .downcast_ref::<arrow_array::Float32Array>()
                        .unwrap();
                    let ptr = float32_data.values().as_ptr();
                    unsafe {
                        ffi_internal::fill_column_f32(pcol.column.clone(), ptr, start_at, col.len())
                    };
                }
                DataType::Float64 => {
                    let float64_data = col
                        .as_any()
                        .downcast_ref::<arrow_array::Float64Array>()
                        .unwrap();
                    let ptr = float64_data.values().as_ptr();
                    unsafe {
                        ffi_internal::fill_column_f64(pcol.column.clone(), ptr, start_at, col.len())
                    };
                }
                DataType::Timestamp(_, _) => todo!(),
                DataType::Date32 => {
                    let date32_data = col
                        .as_any()
                        .downcast_ref::<arrow_array::Date32Array>()
                        .unwrap();
                    let ptr = date32_data.values().as_ptr();
                    unsafe {
                        ffi_internal::fill_column_date(
                            pcol.column.clone(),
                            ptr,
                            start_at,
                            col.len(),
                        )
                    };
                }
                DataType::Date64 => {
                    let date64_data = col
                        .as_any()
                        .downcast_ref::<arrow_array::Date64Array>()
                        .unwrap();
                    let ptr = date64_data.values().as_ptr();
                    unsafe {
                        ffi_internal::fill_column_time(
                            pcol.column.clone(),
                            ptr,
                            start_at,
                            col.len(),
                        )
                    };
                }
                DataType::Time32(units) => {
                    let time32_data = match units {
                        TimeUnit::Second => col
                            .as_any()
                            .downcast_ref::<arrow_array::Time32SecondArray>()
                            .unwrap()
                            .values(),
                        TimeUnit::Millisecond => col
                            .as_any()
                            .downcast_ref::<arrow_array::Time32MillisecondArray>()
                            .unwrap()
                            .values(),
                        TimeUnit::Microsecond => return mk_err(),
                        TimeUnit::Nanosecond => return mk_err(),
                    };
                    let ptr = time32_data.as_ptr();
                    unsafe {
                        ffi_internal::fill_column_i32(pcol.column.clone(), ptr, start_at, col.len())
                    };
                }
                DataType::Time64(units) => {
                    let time64_data = match units {
                        TimeUnit::Microsecond => col
                            .as_any()
                            .downcast_ref::<arrow_array::Time64MicrosecondArray>()
                            .unwrap()
                            .values(),
                        TimeUnit::Nanosecond => col
                            .as_any()
                            .downcast_ref::<arrow_array::Time64NanosecondArray>()
                            .unwrap()
                            .values(),
                        TimeUnit::Second => return mk_err(),
                        TimeUnit::Millisecond => return mk_err(),
                    };
                    let ptr = time64_data.as_ptr();
                    unsafe {
                        ffi_internal::fill_column_i64(pcol.column.clone(), ptr, start_at, col.len())
                    };
                }
                DataType::Duration(_) => return mk_err(),
                DataType::Interval(_) => return mk_err(),
                DataType::Binary => return mk_err(),
                DataType::FixedSizeBinary(_) => return mk_err(),
                DataType::LargeBinary => return mk_err(),
                DataType::Utf8 => return mk_err(),
                DataType::LargeUtf8 => return mk_err(),
                DataType::List(_) => return mk_err(),
                DataType::FixedSizeList(_, _) => return mk_err(),
                DataType::LargeList(_) => return mk_err(),
                DataType::Struct(_) => return mk_err(),
                DataType::Union(_, _) => return mk_err(),
                arrow_schema::DataType::Dictionary(k, v) => match (k.as_ref(), v.as_ref()) {
                    (&DataType::Int32, &DataType::Utf8) => {
                        let dict_data = col
                            .as_any()
                            .downcast_ref::<arrow_array::DictionaryArray<arrow_array::types::Int32Type>>()
                            .unwrap();
                        let keys = dict_data.keys();
                        let values = dict_data
                            .values()
                            .as_any()
                            .downcast_ref::<arrow_array::StringArray>()
                            .unwrap();
                        let strings = values.values().as_ptr() as *const std::ffi::c_char;
                        let offsets = values.value_offsets();
                        unsafe {
                            // TODO: not sure if this works for all cases
                            //       currently it makes the assumption that all
                            //       record batches share the same dictionary.
                            ffi_internal::fill_column_dict(
                                pcol.column.clone(),
                                strings,
                                offsets,
                                keys.values().as_ptr(),
                                start_at,
                                col.len(),
                            )
                        };
                    }
                    _ => return mk_err(),
                },
                DataType::Decimal128(_, _) => return mk_err(),
                DataType::Decimal256(_, _) => return mk_err(),
                DataType::Map(_, _) => return mk_err(),
                DataType::RunEndEncoded(_, _) => return mk_err(),
            }
        }

        start_at += batch.num_rows();
    }
    let_cxx_string!(index = "psp_pkey");
    Ok(Table {
        table: ffi_internal::mk_table_from_data_table(data_table.data_table, &index),
    })
}
