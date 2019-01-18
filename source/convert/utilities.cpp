#include "ast_converter.hpp"

#include <waveguide/intermediate/data_type.hpp>

namespace waveguide {
namespace ast {

void ast_converter::on_start() const {
    data->current_scope = std::make_shared<intr::scope>();
    data->current_scope->set_debug_label("!ROOT");
    blt()->add_to_scope(data->current_scope);
}

intr::scope_ptr ast_converter::get_result() const {
    return data->current_scope;
}

access_result ast_converter::find_access_result(
    ast::variable_expression const& expr) const {
    auto root_val{lookup_var(expr.name)};
    if (!root_val) {
        throw convert::ast_conversion_exception{
            "There is no variable with name '" + expr.name + "'."
        };
    }
    auto offset{std::make_shared<intr::value>(blt()->INT)};

    if (expr.array_accesses.size() == 0) {
        *offset->data_as_int() = 0;
        offset->set_value_known(true);
    } else {
        auto set{std::make_shared<intr::command>(blt()->COPY)};
        set->add_input(int_literal(0));
        set->add_output(offset);
        add_command(set);
        declare_temp_var(offset);
    }

    auto data_type = root_val->get_type();
	// TODO: Optimize this for multiple sucessive array indexing operations
	// TODO: Add in member access operations once objects are a thing
	// TODO: Add errors if array access or member access is used on an unsupported data type.
	for (auto const&index : expr.array_accesses) {
        auto element_type = std::static_pointer_cast
            <const intr::array_data_type>(data_type)->get_element_type();
        recurse(index);
        intr::value_ptr index_value{data->current_value};

        auto mul{std::make_shared<intr::command>(blt()->MUL)};
        mul->add_input(index_value);
        mul->add_input(int_literal(element_type->get_length()));
        auto mindex{std::make_shared<intr::value>(blt()->INT)};
        declare_temp_var(mindex);
        mul->add_output(mindex);
        add_command(mul);

        auto add{std::make_shared<intr::command>(blt()->ADD)};
        add->add_input(offset);
        add->add_input(mindex);
        add->add_output(offset);
        data_type = element_type;
        add_command(add);
    }

    access_result tr;
    tr.final_type = data_type;
    tr.root_val = root_val;
    tr.offset = offset;
    return tr;
}

void ast_converter::copy_value_to_expr(intr::value_ptr from,
    ast::variable_expression const& to) const {
    auto access = find_access_result(to);
    auto copy{std::make_shared<intr::command>(blt()->COPY)};
    copy->add_input(from);
    copy->add_input(access.offset);
    copy->add_output(access.root_val);
    add_command(copy);
}

void ast_converter::copy_value_from_expr(ast::variable_expression const& from,
    intr::value_ptr to) const {
    auto access = find_access_result(from);
    auto copy{std::make_shared<intr::command>(blt()->COPY)};
    copy->add_input(access.root_val);
    copy->add_input(access.offset);
    copy->add_output(to);
    add_command(copy);
}

intr::value_ptr ast_converter::lookup_var(std::string name) const {
    return data->current_scope->lookup_var(name);
}

intr::scope_ptr ast_converter::lookup_func(std::string name) const {
    return data->current_scope->lookup_func(name);
}

intr::data_type_ptr ast_converter::lookup_type(std::string name) const {
    return data->current_scope->lookup_type(name);
}

void ast_converter::add_command(intr::command_ptr command) const {
    data->current_scope->add_command(command);
}

void ast_converter::declare_temp_var(intr::value_ptr var) const {
    data->current_scope->declare_temp_var(var);
}

}
}