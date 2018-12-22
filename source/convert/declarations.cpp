#include "ast_converter.hpp"

namespace waveguide {
namespace ast {

void AstConverter::operator()(function_parameter_dec const&dec) const {
    // TODO: add logic.
}

void AstConverter::operator()(function_dec const&dec) const {
    auto old_scope = data->current_scope;
    data->current_scope = SP<intr::scope>{new intr::scope{old_scope}};
    // TODO: add logic for inputs/outputs.
    recurse(dec.body);
    intr::command_lambda lambda;
    lambda.name = dec.name;
    lambda.body = data->current_scope;
    old_scope->declare_temp_func(lambda.body);

    old_scope->get_commands().back()->add_lambda(lambda);
    data->current_scope = old_scope;
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