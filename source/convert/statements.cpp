#include "ast_converter.hpp"

#include <waveguide/intermediate/builtins.hpp>

namespace waveguide {
namespace ast {

void ast_converter::operator()(std::vector<statement> const&stats) const {
    for (auto const&stat : stats) {
        recurse(stat);
    }
}

void ast_converter::operator()(function_statement const&stat) const {
    recurse(stat.func_call);
}

void ast_converter::operator()(assign_statement const&stat) const {
    auto copy{std::make_shared<intr::command>(blt()->COPY)};
    recurse(stat.value);
    copy->add_input(data->current_value);
    recurse(stat.assign_to);
    copy->add_output(data->current_value);
    add_command(copy);
}

void ast_converter::operator()(var_dec_statement const&dec) const {
    recurse(dec.type);
    for (auto const&var_dec : dec.var_decs) {
        recurse(var_dec);
    }
}

void ast_converter::operator()(plain_var_dec const&dec) const {
    auto value{std::make_shared<intr::value>(data->current_type)};
    data->current_scope->declare_var(dec.name, value);
}

void ast_converter::operator()(init_var_dec const&dec) const {
    auto value{std::make_shared<intr::value>(data->current_type)};
    data->current_scope->declare_var(dec.name, value);

    auto copy{std::make_shared<intr::command>(blt()->COPY)};
    recurse(dec.value);
    copy->add_input(data->current_value);
    copy->add_output(access(value));
    add_command(copy);
}

void ast_converter::operator()(return_statement const&stat) const {
    auto copy{std::make_shared<intr::command>(blt()->COPY)};
    recurse(stat.value);
    copy->add_input(data->current_value);
    copy->add_output(access(lookup_var("return")));
    add_command(copy);

    auto ret{std::make_shared<intr::command>(blt()->RETURN)};
    add_command(ret);
}

}
}