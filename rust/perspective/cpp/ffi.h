#pragma once

#include "types.h"
#include <perspective/pool.h>
#include <perspective/table.h>
#include <perspective/gnode.h>
#include <perspective/data_table.h>
#include <perspective/column.h>
#include <memory>
#include <perspective/src/ffi.rs.h>

namespace ffi {
std::unique_ptr<Pool> mk_pool();

DType get_col_dtype(const Column& col);

std::shared_ptr<Table> mk_table(rust::Vec<rust::String> column_names_ptr,
    rust::Vec<DType> data_types_ptr, std::uint32_t limit,
    rust::String index_ptr);

} // namespace ffi