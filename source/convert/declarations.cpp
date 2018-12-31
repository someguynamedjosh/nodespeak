#include "ast_converter.hpp"

namespace waveguide {
namespace ast {

void ast_converter::operator()(function_parameter_dec const&dec) const {
    recurse(dec.type);
    SP<intr::data_type> placeholder_type{
        new intr::unresolved_vague_type{data->current_vtype}
    };
    SP<intr::value> placeholder_value;
    if (data->fpd_is_input) {
        placeholder_value = 
            data->current_scope->add_input(dec.name, data->current_vtype);
    } else {
        placeholder_value = 
            data->current_scope->add_output(dec.name, data->current_vtype);
    }
    data->current_scope->declare_var(dec.name, placeholder_value);

    std::vector<std::string> new_vars{}, new_types{};
    data->current_vtype->collect_new_vars(new_vars);
    data->current_vtype->collect_new_types(new_types);
    for (auto var_name : new_vars) {
        if (!data->current_scope->lookup_var(var_name, false)) {
            data->current_scope->declare_var(var_name, SP<intr::value>{
                new intr::value{blt()->INT}
            });
        }
    }
    for (auto type_name : new_types) {
        if (!data->current_scope->lookup_type(type_name, false)) {
            data->current_scope->declare_type(type_name, SP<intr::data_type>{
                new intr::unresolved_vague_type{SP<intr::vague_data_type>{
                    new intr::vague_basic_data_type{type_name}
                }}
            });
        }
    }
}

void ast_converter::operator()(function_dec const&dec) const {
    auto old_scope = data->current_scope;
    data->current_scope = SP<intr::scope>{new intr::scope{old_scope}};
    data->fpd_is_input = true;
    for (auto fpd : dec.inputs) {
        recurse(fpd);
    }
    data->fpd_is_input = false;
    for (auto fpd : dec.outputs) {
        recurse(fpd);
    }
    recurse(dec.body);
    intr::command_lambda lambda;
    lambda.name = dec.name;
    lambda.body = data->current_scope;
    old_scope->declare_temp_func(lambda.body);

    old_scope->get_commands().back()->add_lambda(lambda);
    data->current_scope = old_scope;
}

void ast_converter::operator()(data_type const&type) const {
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

void ast_converter::operator()(vague_data_type const&type) const {
    if (type.is_unknown) {
        data->current_vtype = SP<intr::vague_data_type>{
            new intr::vague_basic_data_type{type.name}
        };
    } else {
        data->current_vtype = SP<intr::vague_data_type>{
            new intr::vague_known_data_type{lookup_type(type.name)}
        };
    }
    for (auto size : type.array_sizes) { 
        recurse(size);
        data->current_vtype = SP<intr::vague_data_type>{
            new intr::vague_array_data_type{
                data->current_vtype, data->current_vexpr
            }
        };
    }
}

void ast_converter::operator()(vague_number_expression const&expr) const {
    data->current_vexpr = SP<intr::vague_expression>{
        new intr::vague_number_expression{expr.value}
    };
}

void ast_converter::operator()(vague_variable_expression const&expr) const {
    if (expr.is_unknown) {
        data->current_vexpr = SP<intr::vague_expression>{
            new intr::vague_value_expression{expr.name}
        };
    } else {
        data->current_vexpr = SP<intr::vague_expression>{
            new intr::vague_known_value_expression{lookup_var(expr.name)}
        };
    }
}

void ast_converter::operator()(vague_signed_expression const&expr) const {
    if (expr.sign == '-') {
        data->current_vexpr = SP<intr::vague_expression>{
            new intr::vague_negation_expression{data->current_vexpr}
        };
    }
}

void ast_converter::operator()(vague_operator_list_expression const&expr) const {
    recurse(expr.start_value);
    for (auto operation : expr.operations) {
        auto old_vexpr = data->current_vexpr;
        recurse(operation.value);
        if (operation.op_char == "+") {
            data->current_vexpr = SP<intr::vague_expression>{
                new intr::vague_add_expression{old_vexpr, data->current_vexpr}
            };
        } else if (operation.op_char == "-") {
            data->current_vexpr = SP<intr::vague_expression>{
                new intr::vague_subtract_expression{
                    old_vexpr, data->current_vexpr
                }
            };
        } else if (operation.op_char == "*") {
            data->current_vexpr = SP<intr::vague_expression>{
                new intr::vague_multiply_expression{
                    old_vexpr, data->current_vexpr
                }
            };
        } else if (operation.op_char == "/") {
            data->current_vexpr = SP<intr::vague_expression>{
                new intr::vague_divide_expression{
                    old_vexpr, data->current_vexpr
                }
            };
        }
    }
}

}
}