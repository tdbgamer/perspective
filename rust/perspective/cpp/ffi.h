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
uint32_t get_col_nth_u32(const Column& col, perspective::t_uindex idx);
uint64_t get_col_nth_u64(const Column& col, perspective::t_uindex idx);
int32_t get_col_nth_i32(const Column& col, perspective::t_uindex idx);
int64_t get_col_nth_i64(const Column& col, perspective::t_uindex idx);
float get_col_nth_f32(const Column& col, perspective::t_uindex idx);
double get_col_nth_f64(const Column& col, perspective::t_uindex idx);

rust::String pretty_print(
    const perspective::Table& table, std::uint32_t num_rows);

std::shared_ptr<Table> mk_table(rust::Vec<rust::String> column_names_ptr,
    rust::Vec<DType> data_types_ptr, std::uint32_t limit,
    rust::String index_ptr);

} // namespace ffi