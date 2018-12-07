#include "ast_converter.hpp"

#include <waveguide/intermediate/data_type.hpp>

namespace waveguide {
namespace ast {

void AstConverter::on_start() const {
    data->current_scope = SP<intr::scope>{new intr::scope()};
    blt()->add_to_scope(data->current_scope);
}

SP<intr::scope> AstConverter::get_result() const {
    return data->current_scope;
}

AccessResult AstConverter::find_access_result(
    ast::variable_expression const& expr) const {
    SP<intr::value> root_val{lookup_var(expr.name)};
    SP<intr::value> offset{new intr::value(blt()->INT)};

    if (expr.array_accesses.size() == 0) {
        *offset->data_as_int() = 0;
        offset->set_value_known(true);
    } else {
        SP<intr::command> set{new intr::command(blt()->COPY)};
        set->add_input(int_literal(0));
        set->add_input(int_literal(0));
        set->add_output(offset);
        add_command(set);
        declare_temp_var(offset);
    }

    SP<intr::data_type> data_type{root_val->get_type()};
	// TODO: Optimize this for multiple sucessive array indexing operations
	// TODO: Add in member access operations once objects are a thing
	// TODO: Add errors if array access or member access is used on an unsupported data type.
	for (auto const&index : expr.array_accesses) {
        SP<intr::data_type> element_type = std::static_pointer_cast
            <intr::array_data_type>(data_type)->get_element_type();
        recurse(index);
        SP<intr::value> index_value{data->current_value};

        SP<intr::command> mul{new intr::command{blt()->MUL}};
        mul->add_input(index_value);
        mul->add_input(int_literal(element_type->get_length()));
        SP<intr::value> mindex{new intr::value{blt()->INT}};
        declare_temp_var(mindex);
        mul->add_output(mindex);
        add_command(mul);

        SP<intr::command> add{new intr::command{blt()->ADD}};
        add->add_input(offset);
        add->add_input(mindex);
        add->add_output(offset);
        data_type = element_type;
        add_command(add);
    }

    AccessResult tr;
    tr.final_type = data_type;
    tr.root_val = root_val;
    tr.offset = offset;
    return tr;
}

void AstConverter::copy_value_to_expr(SP<intr::value> from,
    ast::variable_expression const& to) const {
    auto access = find_access_result(to);
    SP<intr::command> copy{new intr::command(blt()->COPY)};
    copy->add_input(from);
    copy->add_input(access.offset);
    copy->add_output(access.root_val);
    add_command(copy);
}

void AstConverter::copy_value_from_expr(ast::variable_expression const& from,
    SP<intr::value> to) const {
    auto access = find_access_result(from);
    SP<intr::command> copy{new intr::command(blt()->COPY)};
    copy->add_input(access.root_val);
    copy->add_input(access.offset);
    copy->add_output(to);
    add_command(copy);
}

SP<intr::value> AstConverter::lookup_var(std::string name) const {
    return data->current_scope->lookup_var(name);
}

SP<intr::scope> AstConverter::lookup_func(std::string name) const {
    return data->current_scope->lookup_func(name);
}

SP<intr::data_type> AstConverter::lookup_type(std::string name) const {
    return data->current_scope->lookup_type(name);
}

void AstConverter::add_command(SP<intr::command> command) const {
    data->current_scope->add_command(command);
}

void AstConverter::declare_temp_var(SP<intr::value> var) const {
    data->current_scope->declare_temp_var(var);
}

}
}