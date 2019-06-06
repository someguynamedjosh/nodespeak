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