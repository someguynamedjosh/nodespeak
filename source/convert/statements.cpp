#include "ast_converter.hpp"

#include "intermediate/builtins.hpp"

namespace waveguide {
namespace ast {

void AstConverter::operator()(std::vector<Statement> const&stats) const {
    for (auto const&stat : stats) {
        recurse(stat);
    }
}

void AstConverter::operator()(FunctionStatement const&stat) const {
    recurse(stat.func_call);
}

void AstConverter::operator()(AssignStatement const&stat) const {
    recurse(stat.value);
    copy_value_to_expr(data->current_value, stat.assign_to);
}

void AstConverter::operator()(VarDecStatement const&dec) const {
    recurse(dec.type);
    for (auto const&var_dec : dec.var_decs) {
        recurse(var_dec);
    }
}

void AstConverter::operator()(PlainVarDec const&dec) const {
    SP<intr::Value> value{new intr::Value{data->current_type}};
    data->current_scope->declare_var(dec.name, value);
}

void AstConverter::operator()(InitVarDec const&dec) const {
    SP<intr::Value> value{new intr::Value{data->current_type}};
    data->current_scope->declare_var(dec.name, value);

    SP<intr::Command> copy{new intr::Command{blt()->COPY}};
    recurse(dec.value);
    copy->add_input(data->current_value);
    copy->add_input(int_literal(0));
    copy->add_output(value);
    add_command(copy);
}

void AstConverter::operator()(ReturnStatement const&stat) const {
    SP<intr::Command> copy{new intr::Command{blt()->COPY}};
    recurse(stat.value);
    copy->add_input(data->current_value);
    copy->add_input(int_literal(0));
    copy->add_output(lookup_var("return"));
    add_command(copy);

    SP<intr::Command> ret{new intr::Command{blt()->RETURN}};
    add_command(ret);
}

}
}