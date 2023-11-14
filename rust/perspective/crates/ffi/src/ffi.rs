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

        pub fn get_dtype_size(dtype: DType) -> usize;

        pub unsafe fn get_col_raw_data(col: &Column) -> *mut c_char;

        pub unsafe fn fill_column_memcpy(
            col: SharedPtr<Column>,
            ptr: *const c_char,
            start: usize,
            len: usize,
            size: usize,
        );

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

    pub fn get_dtype_size(&self) -> usize {
        ffi_internal::get_dtype_size(self.get_dtype())
    }

    pub fn get_u32(&self, idx: usize) -> u32 {
        ffi_internal::get_col_nth_u32(&self.column, idx)
    }

    pub fn get_u64(&self, idx: usize) -> u64 {
        ffi_internal::get_col_nth_u64(&self.column, idx)
    }

    pub(crate) unsafe fn get_raw_data(&self) -> *mut std::ffi::c_char {
        ffi_internal::get_col_raw_data(&self.column)
    }

    pub fn as_slice(&self) -> &[u8] {
        let ptr = unsafe { self.get_raw_data() };
        let len = self.size() * self.get_dtype_size();
        unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }
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

impl TryFrom<Schema> for arrow_schema::Schema {
    type Error = perspective_api::Error;

    fn try_from(value: Schema) -> Result<Self, Self::Error> {
        let columns = value.columns();
        let types = value.types();
        let mut fields = Vec::new();
        for (name, dtype) in columns.iter().zip(types.iter()) {
            let field = match *dtype {
                DType::DTYPE_INT32 => {
                    arrow_schema::Field::new(name, arrow_schema::DataType::Int32, true)
                }
                DType::DTYPE_INT64 => {
                    arrow_schema::Field::new(name, arrow_schema::DataType::Int64, true)
                }
                DType::DTYPE_UINT32 => {
                    arrow_schema::Field::new(name, arrow_schema::DataType::UInt32, true)
                }
                DType::DTYPE_UINT64 => {
                    arrow_schema::Field::new(name, arrow_schema::DataType::UInt64, true)
                }
                DType::DTYPE_FLOAT32 => {
                    arrow_schema::Field::new(name, arrow_schema::DataType::Float32, true)
                }
                DType::DTYPE_FLOAT64 => {
                    arrow_schema::Field::new(name, arrow_schema::DataType::Float64, true)
                }
                DType::DTYPE_DATE => {
                    arrow_schema::Field::new(name, arrow_schema::DataType::Date32, true)
                }
                DType::DTYPE_TIME => {
                    arrow_schema::Field::new(name, arrow_schema::DataType::Date64, true)
                }
                DType::DTYPE_STR => arrow_schema::Field::new(
                    name,
                    arrow_schema::DataType::Dictionary(
                        Box::new(arrow_schema::DataType::Int32),
                        Box::new(arrow_schema::DataType::Utf8),
                    ),
                    true,
                ),
                _ => {
                    return Err(perspective_api::Error::UnsupportedArrowSchema(format!(
                        "{:?}",
                        dtype
                    )))
                }
            };
            fields.push(field);
        }
        Ok(arrow_schema::Schema::new(fields))
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

pub fn write_arrow(table: &Table) -> perspective_api::Result<Vec<u8>> {
    let arrow_schema = Arc::new(arrow_schema::Schema::try_from(table.schema())?);
    let mut builder = arrow_ipc::writer::StreamWriter::try_new(Vec::new(), &arrow_schema).unwrap();
    let schema = table.schema();
    let cols = schema
        .columns()
        .into_iter()
        .zip(schema.types().into_iter())
        .collect::<Vec<_>>();
    let mut array_refs: Vec<(String, arrow_array::ArrayRef, bool)> = Vec::new();
    for (col_name, col_dtype) in cols {
        let col = table.get_column(&col_name);
        let buffer = arrow_buffer::Buffer::from_slice_ref(col.as_slice());
        match col_dtype {
            DType::DTYPE_INT32 => {
                let scalar_buffer = arrow_buffer::ScalarBuffer::from(buffer);
                let array = arrow_array::Int32Array::new(scalar_buffer, None);
                // let array = arrow_array::Int32Array::(buffer);
                array_refs.push((col_name, Arc::new(array), true));
            }
            DType::DTYPE_INT64 => {
                let scalar_buffer = arrow_buffer::ScalarBuffer::from(buffer);
                let array = arrow_array::Int64Array::new(scalar_buffer, None);
                array_refs.push((col_name, Arc::new(array), true));
            }
            DType::DTYPE_UINT32 => {
                let scalar_buffer = arrow_buffer::ScalarBuffer::from(buffer);
                let array = arrow_array::UInt32Array::new(scalar_buffer, None);
                array_refs.push((col_name, Arc::new(array), true));
            }
            DType::DTYPE_UINT64 => {
                let scalar_buffer = arrow_buffer::ScalarBuffer::from(buffer);
                let array = arrow_array::UInt64Array::new(scalar_buffer, None);
                array_refs.push((col_name, Arc::new(array), true));
            }
            DType::DTYPE_FLOAT32 => {
                let scalar_buffer = arrow_buffer::ScalarBuffer::from(buffer);
                let array = arrow_array::Float32Array::new(scalar_buffer, None);
                array_refs.push((col_name, Arc::new(array), true));
            }
            DType::DTYPE_FLOAT64 => {
                let scalar_buffer = arrow_buffer::ScalarBuffer::from(buffer);
                let array = arrow_array::Float64Array::new(scalar_buffer, None);
                array_refs.push((col_name, Arc::new(array), true));
            }
            _ => todo!(),
        }
    }
    // let b = arrow_array::RecordBatch::try_new(arrow_schema.clone(), array_refs).unwrap();

    let batch = arrow_array::RecordBatch::try_from_iter_with_nullable(array_refs).unwrap();
    builder.write(&batch).unwrap();
    builder.finish().unwrap();
    Ok(builder.into_inner().unwrap())
    // arrow_array::Int32Array::new(values, nulls)
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
                        // ffi_internal::fill_column_i32(pcol.column.clone(), ptr, start_at, col.len())
                        ffi_internal::fill_column_memcpy(
                            pcol.column.clone(),
                            ptr as *const std::ffi::c_char,
                            start_at,
                            col.len(),
                            std::mem::size_of::<i32>(),
                        )
                    };
                }
                DataType::Int64 => {
                    let int64_data = col
                        .as_any()
                        .downcast_ref::<arrow_array::Int64Array>()
                        .unwrap();
                    let ptr = int64_data.values().as_ptr();
                    unsafe {
                        // ffi_internal::fill_column_i64(pcol.column.clone(), ptr, start_at, col.len())
                        ffi_internal::fill_column_memcpy(
                            pcol.column.clone(),
                            ptr as *const std::ffi::c_char,
                            start_at,
                            col.len(),
                            std::mem::size_of::<i64>(),
                        )
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
                        // ffi_internal::fill_column_u32(pcol.column.clone(), ptr, start_at, col.len())
                        ffi_internal::fill_column_memcpy(
                            pcol.column.clone(),
                            ptr as *const std::ffi::c_char,
                            start_at,
                            col.len(),
                            std::mem::size_of::<u32>(),
                        )
                    };
                }
                DataType::UInt64 => {
                    let uint64_data = col
                        .as_any()
                        .downcast_ref::<arrow_array::UInt64Array>()
                        .unwrap();
                    let ptr = uint64_data.values().as_ptr();
                    unsafe {
                        // ffi_internal::fill_column_u64(pcol.column.clone(), ptr, start_at, col.len())
                        ffi_internal::fill_column_memcpy(
                            pcol.column.clone(),
                            ptr as *const std::ffi::c_char,
                            start_at,
                            col.len(),
                            std::mem::size_of::<u64>(),
                        )
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
                        // ffi_internal::fill_column_f32(pcol.column.clone(), ptr, start_at, col.len())
                        ffi_internal::fill_column_memcpy(
                            pcol.column.clone(),
                            ptr as *const std::ffi::c_char,
                            start_at,
                            col.len(),
                            std::mem::size_of::<f32>(),
                        )
                    };
                }
                DataType::Float64 => {
                    let float64_data = col
                        .as_any()
                        .downcast_ref::<arrow_array::Float64Array>()
                        .unwrap();
                    let ptr = float64_data.values().as_ptr();
                    unsafe {
                        // ffi_internal::fill_column_f64(pcol.column.clone(), ptr, start_at, col.len())
                        ffi_internal::fill_column_memcpy(
                            pcol.column.clone(),
                            ptr as *const std::ffi::c_char,
                            start_at,
                            col.len(),
                            std::mem::size_of::<f64>(),
                        )
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
                        // ffi_internal::fill_column_i32(pcol.column.clone(), ptr, start_at, col.len())
                        ffi_internal::fill_column_memcpy(
                            pcol.column.clone(),
                            ptr as *const std::ffi::c_char,
                            start_at,
                            col.len(),
                            std::mem::size_of::<i32>(),
                        )
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
                        // ffi_internal::fill_column_i64(pcol.column.clone(), ptr, start_at, col.len())
                        ffi_internal::fill_column_memcpy(
                            pcol.column.clone(),
                            ptr as *const std::ffi::c_char,
                            start_at,
                            col.len(),
                            std::mem::size_of::<i64>(),
                        )
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

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use arrow_array::{Array, ArrayRef, RecordBatch};

    #[test]
    pub fn test_discover_arrow_behavior() {
        // Ensuring my assumptions about arrow's behavior are correct.
        let i32a = arrow_array::Int32Array::from(vec![Some(1), Some(2), Some(3), Some(4), None]);
        assert_eq!(i32a.len(), 5);
        assert_eq!(i32a.null_count(), 1);

        // wtf!
        assert_eq!(i32a.nulls().unwrap().len(), 5);
        // len on nulls is the same as len on the array, but null_count is correct...
        assert_eq!(i32a.nulls().unwrap().null_count(), 1);

        // Includes nulls *sigh*
        assert_eq!(i32a.values().len(), 5);

        // Obviously should be unchanged
        assert_eq!(i32a.values().slice(0, 4), &[1, 2, 3, 4]);
        // Ok looks like it just leaves the value uninitialized (I assume they didn't zero it)
        assert_eq!(i32a.value(4), 0);

        // This may not be so bad then. I can memset the column with the values and then
        // memset the statuses using the null buffer.
    }

    #[test]
    pub fn test_read_write_arrow() {
        let arrow = arrow_array::RecordBatch::try_from_iter(vec![
            (
                "a",
                Arc::new(arrow_array::Int32Array::from(vec![
                    Some(1),
                    Some(2),
                    Some(3),
                    Some(4),
                    // None,
                ])) as ArrayRef,
            ),
            (
                "b",
                Arc::new(arrow_array::Int32Array::from(vec![
                    Some(1),
                    Some(2),
                    Some(3),
                    Some(4),
                    // None,
                ])) as ArrayRef,
            ),
        ])
        .unwrap();
        let mut writer =
            arrow_ipc::writer::StreamWriter::try_new(Vec::new(), &arrow.schema()).unwrap();
        writer.write(&arrow).unwrap();
        writer.finish().unwrap();
        let raw_arroy_bytes = writer.into_inner().unwrap();
        let table = read_arrow(raw_arroy_bytes.as_slice()).unwrap();
        let psp_arrow_bytes = write_arrow(&table).unwrap();
        let _ = read_arrow(psp_arrow_bytes.as_slice()).unwrap();
        assert_arrow_arrays_same(raw_arroy_bytes.as_slice(), psp_arrow_bytes.as_slice());
    }

    pub fn assert_arrow_arrays_same(lhs: &[u8], rhs: &[u8]) {
        let lhs = arrow_ipc::reader::StreamReader::try_new(std::io::Cursor::new(lhs), None)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        let rhs = arrow_ipc::reader::StreamReader::try_new(std::io::Cursor::new(rhs), None)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        // Ignore missing fields for now
        let fields_lhs = lhs
            .schema()
            .all_fields()
            .iter()
            .map(|x| x.name())
            .cloned()
            .collect::<HashSet<_>>();
        let fields_rhs = rhs
            .schema()
            .all_fields()
            .iter()
            .map(|x| x.name())
            .cloned()
            .collect::<HashSet<_>>();
        let fields = fields_lhs.intersection(&fields_rhs).collect::<Vec<_>>();

        for field in fields {
            let (col_l, col_r) = (
                lhs.column_by_name(field).unwrap(),
                rhs.column_by_name(field).unwrap(),
            );
            assert_eq!(col_l.data_type(), col_r.data_type());
            assert_eq!(col_l.len(), col_r.len());
            assert_eq!(col_l.null_count(), col_r.null_count());
            assert_eq!(col_l.nulls(), col_r.nulls());
            assert_eq!(col_l.to_data(), col_r.to_data());
        }
    }
}
