#[cxx::bridge(namespace = ffi)]
pub mod ffi_common {
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
}

pub use ffi_common::DType;
