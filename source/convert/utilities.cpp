#include "ast_converter.hpp"

#include "intermediate/data_type.hpp"

namespace waveguide {
namespace ast {

AstConverter::AstConverter(): data{new ConverterData{}} {
    data->current_scope = SP<intr::Scope>{new intr::Scope()};
    blt()->add_to_scope(data->current_scope);
}

void AstConverter::push_stack() const {
    stack.push_back(SP<ConverterData>{new ConverterData{*data}});
}

void AstConverter::pop_stack() const {
    auto popped = stack.back();
    data->current_scope = popped->current_scope;
    data->current_value = popped->current_value;
    data->current_type = popped->current_type;
    stack.pop_back();
}

AccessResult AstConverter::find_access_result(
    ast::VariableExpression const& expr) const {
    SP<intr::Value> root_val{lookup_var(expr.name)};
    SP<intr::Value> offset{new intr::Value(blt()->INT)};

    if (expr.array_accesses.size() == 0) {
        *offset->data_as_int() = 0;
        offset->set_value_known(true);
    } else {
        SP<intr::Command> set{new intr::Command(blt()->COPY)};
        set->add_input(int_literal(0));
        set->add_input(int_literal(0));
        set->add_output(offset);
        add_command(set);
        declare_temp_var(offset);
    }

    SP<intr::DataType> data_type{root_val->get_type()};
	// TODO: Optimize this for multiple sucessive array indexing operations
	// TODO: Add in member access operations once objects are a thing
	// TODO: Add errors if array access or member access is used on an unsupported data type.
	for (auto const&index : expr.array_accesses) {
        SP<intr::DataType> element_type = std::static_pointer_cast
            <intr::ArrayDataType>(data_type)->get_element_type();
        recurse(index);
        SP<intr::Value> index_value{data->current_value};

        SP<intr::Command> mul{new intr::Command{blt()->MUL}};
        mul->add_input(index_value);
        mul->add_input(int_literal(element_type->get_length()));
        SP<intr::Value> mindex{new intr::Value{blt()->INT}};
        declare_temp_var(mindex);
        mul->add_output(mindex);
        add_command(mul);

        SP<intr::Command> add{new intr::Command{blt()->ADD}};
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

void AstConverter::copy_value_to_expr(SP<intr::Value> from,
    ast::VariableExpression const& to) const {
    auto access = find_access_result(to);
    SP<intr::Command> copy{new intr::Command(blt()->COPY)};
    copy->add_input(from);
    copy->add_input(access.offset);
    copy->add_output(access.root_val);
    add_command(copy);
}

void AstConverter::copy_value_from_expr(ast::VariableExpression const& from,
    SP<intr::Value> to) const {
    auto access = find_access_result(from);
    SP<intr::Command> copy{new intr::Command(blt()->COPY)};
    copy->add_input(access.root_val);
    copy->add_input(access.offset);
    copy->add_output(to);
    add_command(copy);
}

SP<intr::Value> AstConverter::lookup_var(std::string name) const {
    return data->current_scope->lookup_var(name);
}

SP<intr::Scope> AstConverter::lookup_func(std::string name) const {
    return data->current_scope->lookup_func(name);
}

SP<intr::DataType> AstConverter::lookup_type(std::string name) const {
    return data->current_scope->lookup_type(name);
}

void AstConverter::add_command(SP<intr::Command> command) const {
    data->current_scope->add_command(command);
}

void AstConverter::declare_temp_var(SP<intr::Value> var) const {
    data->current_scope->declare_temp_var(var);
}

}
}