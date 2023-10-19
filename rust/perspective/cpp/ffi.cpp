#include "ffi.h"
#include "perspective/base.h"

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

std::shared_ptr<Table>
mk_table(rust::Vec<rust::String> column_names_ptr,
    rust::Vec<DType> data_types_ptr, std::uint32_t limit,
    rust::String index_ptr) {
    // std::shared_ptr<perspective::t_gnode> gnode;
    // gnode->init();

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

    // tbl->set_gnode(gnode);

    tbl->init(data_table, 0, perspective::t_op::OP_INSERT, 0);
    return tbl;
}
} // namespace ffi