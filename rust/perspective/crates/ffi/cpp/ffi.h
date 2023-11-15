#pragma once

#include "perspective/raw_types.h"
#include "types.h"
#include <perspective/pool.h>
#include <perspective/table.h>
#include <perspective/gnode.h>
#include <perspective/data_table.h>
#include <perspective/column.h>
#include <memory>
#include <rust/cxx.h>

namespace ffi {
std::unique_ptr<Pool> mk_pool();

enum class DType : ::std::uint8_t;
enum class Status : ::std::uint8_t;

DType get_col_dtype(const Column& col);
perspective::t_uindex get_dtype_size(DType dtype);
uint32_t get_col_nth_u32(const Column& col, perspective::t_uindex idx);
uint64_t get_col_nth_u64(const Column& col, perspective::t_uindex idx);
int32_t get_col_nth_i32(const Column& col, perspective::t_uindex idx);
int64_t get_col_nth_i64(const Column& col, perspective::t_uindex idx);
float get_col_nth_f32(const Column& col, perspective::t_uindex idx);
double get_col_nth_f64(const Column& col, perspective::t_uindex idx);

Status* get_col_raw_status(const Column& col);

char* get_col_raw_data(const Column& col);

void fill_column_memcpy(std::shared_ptr<Column> col, const char* ptr,
    const unsigned char* nullmask, perspective::t_uindex start,
    perspective::t_uindex len, std::size_t size);

void fill_column_date(std::shared_ptr<Column> col, const std::int32_t* ptr,
    const unsigned char* nullmask, perspective::t_uindex start,
    perspective::t_uindex len);

void fill_column_time(std::shared_ptr<Column> col, const std::int64_t* ptr,
    const unsigned char* nullmask, perspective::t_uindex start,
    perspective::t_uindex len);

void fill_column_dict(std::shared_ptr<Column> col, const char* dict,
    rust::Slice<const std::int32_t> offsets, const std::int32_t* ptr,
    const unsigned char* nullmask, perspective::t_uindex start,
    perspective::t_uindex len);

perspective::t_uindex make_table_port(const Table& table);

bool process_gnode(const GNode& col, perspective::t_uindex idx);

rust::Vec<rust::String> get_schema_columns(const Schema& schema);
rust::Vec<DType> get_schema_types(const Schema& schema);

std::unique_ptr<Schema> get_table_schema(const Table& table);

rust::String pretty_print(
    const perspective::Table& table, std::size_t num_rows);

std::shared_ptr<Table> mk_table(rust::Vec<rust::String> column_names_ptr,
    rust::Vec<DType> data_types_ptr, std::uint32_t limit,
    rust::String index_ptr);

std::shared_ptr<Table> mk_table_from_data_table(
    std::unique_ptr<DataTable> data_table, const std::string& index);

std::unique_ptr<Schema> mk_schema(
    rust::Vec<rust::String> column_names_ptr, rust::Vec<DType> data_types_ptr);

std::unique_ptr<DataTable> mk_data_table(
    const Schema& schema, perspective::t_uindex capacity);

std::unique_ptr<DataTable> table_extend(
    std::unique_ptr<DataTable> table, perspective::t_uindex num_rows);

} // namespace ffi