#include "ffi.h"
#include "perspective/raw_types.h"
#include "types.h"
#include <memory>
#include <perspective/base.h>
#include <perspective/column.h>
#include <chrono>

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

void
fill_column_u32(std::shared_ptr<Column> col, const std::uint32_t* ptr,
    perspective::t_uindex start, perspective::t_uindex len) {
    for (auto i = 0; i < len; ++i) {
        col->set_nth<std::uint32_t>(start + i, ptr[i]);
    }
};

void
fill_column_u64(std::shared_ptr<Column> col, const std::uint64_t* ptr,
    perspective::t_uindex start, perspective::t_uindex len) {
    for (auto i = 0; i < len; ++i) {
        col->set_nth<std::uint64_t>(start + i, ptr[i]);
    }
};

void
fill_column_i32(std::shared_ptr<Column> col, const std::int32_t* ptr,
    perspective::t_uindex start, perspective::t_uindex len) {
    for (auto i = 0; i < len; ++i) {
        col->set_nth<std::int32_t>(start + i, ptr[i]);
    }
};

void
fill_column_i64(std::shared_ptr<Column> col, const std::int64_t* ptr,
    perspective::t_uindex start, perspective::t_uindex len) {
    for (auto i = 0; i < len; ++i) {
        col->set_nth<std::int64_t>(start + i, ptr[i]);
    }
}

void
fill_column_f32(std::shared_ptr<Column> col, const float* ptr,
    perspective::t_uindex start, perspective::t_uindex len) {
    for (auto i = 0; i < len; ++i) {
        col->set_nth<float>(start + i, ptr[i]);
    }
}

void
fill_column_f64(std::shared_ptr<Column> col, const double* ptr,
    perspective::t_uindex start, perspective::t_uindex len) {
    for (auto i = 0; i < len; ++i) {
        col->set_nth<double>(start + i, ptr[i]);
    }
}

void
fill_column_date(std::shared_ptr<Column> col, const std::int32_t* ptr,
    perspective::t_uindex start, perspective::t_uindex len) {
    for (auto i = 0; i < len; ++i) {
        // Calculate year month and day from
        std::chrono::time_point<std::chrono::system_clock> tp
            = std::chrono::system_clock::from_time_t(
                ptr[i] * 24 * 60 * 60); // days to seconds
        std::time_t time = std::chrono::system_clock::to_time_t(tp);
        std::tm* date = std::localtime(&time);
        perspective::t_date dt(date->tm_year, date->tm_mon, date->tm_mday);
        col->set_nth<perspective::t_date>(start + i, dt);
    }
}

void
fill_column_time(std::shared_ptr<Column> col, const std::int64_t* ptr,
    perspective::t_uindex start, perspective::t_uindex len) {
    for (auto i = 0; i < len; ++i) {
        perspective::t_time time(ptr[i]);
        col->set_nth<perspective::t_time>(start + i, time);
    }
}

void
fill_column_dict(std::shared_ptr<Column> col, const char* dict,
    rust::Slice<const std::int32_t> offsets, const std::int32_t* ptr,
    perspective::t_uindex start, perspective::t_uindex len) {
    perspective::t_vocab* vocab = col->_get_vocab();
    std::string s;
    for (std::size_t i = 0; i < offsets.size() - 1; ++i) {
        std::size_t es = offsets[i + 1] - offsets[i];
        s.assign(dict + offsets[i], es);
        vocab->get_interned(s);
    }
    for (std::size_t i = 0; i < len; ++i) {
        col->set_nth<std::int32_t>(start + i, ptr[i]);
    }
}

perspective::t_uindex
make_table_port(const Table& table) {
    return const_cast<Table&>(table).make_port();
}

rust::String
pretty_print(const perspective::Table& table, std::size_t num_rows) {
    std::stringstream ss;
    table.get_gnode()->get_table()->pprint(num_rows, &ss);
    std::string s = ss.str();
    return rust::String(s);
}

bool
process_gnode(const GNode& gnode, perspective::t_uindex idx) {
    return const_cast<GNode&>(gnode).process(idx);
}

rust::Vec<rust::String>
get_schema_columns(const Schema& schema) {
    rust::Vec<rust::String> columns;
    for (auto& s : schema.columns()) {
        columns.push_back(rust::String(s));
    }
    return columns;
}

rust::Vec<DType>
get_schema_types(const Schema& schema) {
    rust::Vec<DType> types;
    for (auto& s : schema.types()) {
        types.push_back(static_cast<DType>(s));
    }
    return types;
}

std::unique_ptr<Schema>
get_table_schema(const Table& table) {
    return std::make_unique<Schema>(table.get_schema());
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

std::shared_ptr<Table>
mk_table_from_data_table(
    std::unique_ptr<DataTable> data_table, const std::string& index) {
    auto pool = std::make_shared<Pool>();
    auto schema = data_table->get_schema();
    std::shared_ptr<Column> pkey;
    if (schema.has_column("psp_pkey")) {
        pkey = data_table->get_column("psp_pkey");
    } else {
        pkey = data_table->add_column_sptr(
            "psp_pkey", perspective::DTYPE_INT64, true);
        for (std::uint64_t i = 0; i < data_table->size(); ++i) {
            pkey->set_nth<std::uint64_t>(i, i);
        }
    }
    if (!schema.has_column("psp_okey")) {
        data_table->clone_column("psp_pkey", "psp_okey");
    }

    auto columns = data_table->get_schema().columns();
    auto dtypes = data_table->get_schema().types();

    auto tbl = std::make_shared<Table>(
        pool, columns, dtypes, data_table->num_rows(), index);
    tbl->init(
        *data_table, data_table->num_rows(), perspective::t_op::OP_INSERT, 0);
    auto gnode = tbl->get_gnode();
    gnode->process(0);
    return tbl;
}

std::unique_ptr<Schema>
mk_schema(
    rust::Vec<rust::String> column_names_ptr, rust::Vec<DType> data_types_ptr) {
    std::vector<std::string> column_names;
    for (auto s : column_names_ptr) {
        std::string ss(s.begin(), s.end());
        column_names.push_back(ss);
    }
    std::vector<perspective::t_dtype> data_types;
    for (auto s : data_types_ptr) {
        data_types.push_back(convert_to_dtype(s));
    }
    return std::make_unique<Schema>(
        perspective::t_schema(column_names, data_types));
}

std::unique_ptr<DataTable>
mk_data_table(const Schema& schema, perspective::t_uindex capacity) {
    auto data_table = std::make_unique<DataTable>(schema, capacity);
    data_table->init();
    return data_table;
}

std::unique_ptr<DataTable>
table_extend(std::unique_ptr<DataTable> table, perspective::t_uindex num_rows) {
    table->extend(num_rows);
    return table;
}

} // namespace ffi