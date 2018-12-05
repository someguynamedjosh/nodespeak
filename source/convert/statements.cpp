#include "ast_converter.hpp"

#include <waveguide/intermediate/builtins.hpp>

namespace waveguide {
namespace ast {

void AstConverter::operator()(std::vector<statement> const&stats) const {
    for (auto const&stat : stats) {
        recurse(stat);
    }
}

void AstConverter::operator()(function_statement const&stat) const {
    recurse(stat.func_call);
}

void AstConverter::operator()(assign_statement const&stat) const {
    recurse(stat.value);
    copy_value_to_expr(data->current_value, stat.assign_to);
}

void AstConverter::operator()(var_dec_statement const&dec) const {
    recurse(dec.type);
    for (auto const&var_dec : dec.var_decs) {
        recurse(var_dec);
    }
}

void AstConverter::operator()(Plainvar_dec const&dec) const {
    SP<intr::value> value{new intr::value{data->current_type}};
    data->current_scope->declare_var(dec.name, value);
}

void AstConverter::operator()(init_var_dec const&dec) const {
    SP<intr::value> value{new intr::value{data->current_type}};
    data->current_scope->declare_var(dec.name, value);

    SP<intr::command> copy{new intr::command{blt()->COPY}};
    recurse(dec.value);
    copy->add_input(data->current_value);
    copy->add_input(int_literal(0));
    copy->add_output(value);
    add_command(copy);
}

void AstConverter::operator()(return_statement const&stat) const {
    SP<intr::command> copy{new intr::command{blt()->COPY}};
    recurse(stat.value);
    copy->add_input(data->current_value);
    copy->add_input(int_literal(0));
    copy->add_output(lookup_var("return"));
    add_command(copy);

    SP<intr::command> ret{new intr::command{blt()->RETURN}};
    add_command(ret);
}

}
}