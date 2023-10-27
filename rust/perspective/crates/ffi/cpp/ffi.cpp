#include "ffi.h"
#include <perspective/base.h>
#include <perspective/column.h>

namespace ffi {

std::unique_ptr<Pool>
mk_pool() {
    return std::unique_ptr<Pool>(new Pool());
}

perspective::t_dtype
convert_to_dtype(DType dtype) {
    return static_cast<perspective::t_dtype>(dtype);
};

DType
get_col_dtype(const Column& col) {
    return static_cast<DType>(col.get_dtype());
}

uint32_t
get_col_nth_u32(const Column& col, perspective::t_uindex idx) {
    return *col.get_nth<uint32_t>(idx);
}

uint64_t
get_col_nth_u64(const Column& col, perspective::t_uindex idx) {
    return *col.get_nth<uint64_t>(idx);
}

int32_t
get_col_nth_i32(const Column& col, perspective::t_uindex idx) {
    return *col.get_nth<int32_t>(idx);
}

int64_t
get_col_nth_i64(const Column& col, perspective::t_uindex idx) {
    return *col.get_nth<int64_t>(idx);
}

float
get_col_nth_f32(const Column& col, perspective::t_uindex idx) {
    return *col.get_nth<float>(idx);
}

double
get_col_nth_f64(const Column& col, perspective::t_uindex idx) {
    return *col.get_nth<double>(idx);
}

rust::String
pretty_print(const perspective::Table& table, std::size_t num_rows) {
    std::stringstream ss;
    table.get_gnode()->get_table()->pprint(num_rows, &ss);
    std::string s = ss.str();
    return rust::String(s);
}

std::shared_ptr<Table>
mk_table(rust::Vec<rust::String> column_names_ptr,
    rust::Vec<DType> data_types_ptr, std::uint32_t limit,
    rust::String index_ptr) {

    std::vector<std::string> column_names;
    for (auto s : column_names_ptr) {
        std::string ss(s.begin(), s.end());
        column_names.push_back(ss);
    }
    std::vector<perspective::t_dtype> data_types;
    for (auto s : data_types_ptr) {
        data_types.push_back(convert_to_dtype(s));
    }
    std::string index(index_ptr.begin(), index_ptr.end());

    auto pool = std::make_shared<Pool>();
    auto tbl
        = std::make_shared<Table>(pool, column_names, data_types, limit, index);

    perspective::t_schema schema(column_names, data_types);
    perspective::t_data_table data_table(schema);
    data_table.init();

    auto size = 3;

    data_table.extend(size);

    auto col = data_table.get_column("a");
    col->set_nth<int64_t>(0, 0);
    col->set_nth<int64_t>(1, 1);
    col->set_nth<int64_t>(2, 2);
    col->valid_raw_fill();

    data_table.clone_column("a", "psp_pkey");
    data_table.clone_column("psp_pkey", "psp_okey");

    tbl->init(data_table, size, perspective::t_op::OP_INSERT, 0);

    auto gnode = tbl->get_gnode();
    gnode->process(0);

    return tbl;
}
} // namespace ffi