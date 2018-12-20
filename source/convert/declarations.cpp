#include "ast_converter.hpp"

namespace waveguide {
namespace ast {

void AstConverter::operator()(function_parameter_dec const&dec) const {
    // TODO: add logic.
}

void AstConverter::operator()(function_dec const&dec) const {
    // TODO: add logic.
}

void AstConverter::operator()(data_type const&type) const {
    // TODO: add logic for array types.
    data->current_type = lookup_type(type.name);
    for (auto size : type.array_sizes) {
        recurse(size);
        // TODO: Error if value is not known.
        data->current_type = SP<intr::array_data_type>{
            new intr::array_data_type{
                data->current_type, *data->current_value->data_as_int()
            }
        };
    }
}

}
}