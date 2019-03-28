#include <waveguide/intermediate/data_type.hpp>
#include <waveguide/intermediate/scope.hpp>
#include <waveguide/intermediate/value.hpp>
#include <set>
#include <sstream>

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

#pragma GCC diagnostic ignored "-Wunused-parameter"
std::ostream &operator<<(std::ostream &stream, do_if_aug const&to_print) {
    return stream;
}
std::ostream &operator<<(std::ostream &stream, do_if_not_aug const&to_print) {
    return stream;
}
std::ostream &operator<<(std::ostream &stream, loop_for_aug const&to_print) {
    return stream;
}
std::ostream &operator<<(std::ostream &stream, loop_range_aug const&to_print) {
    return stream;
}
#pragma GCC diagnostic pop

void print_value(std::string const&indent, std::ostream &stream, value const&to_print) {
    stream << indent << "Label: " << to_print.get_debug_label() << std::endl;
    stream << indent << "Type: " << to_print.get_type() << " (";
    to_print.get_type()->print_repr(stream);
    stream << ")" << std::endl;
    if (to_print.is_value_known()) {
        stream << indent << "Value: ";
        to_print.get_type()->format(stream, to_print.get_data());
        stream << std::endl;
    }
    if (to_print.is_proxy()) {
        stream << indent << "Proxy for: ";
        stream << &to_print.get_real_value() << std::endl;
    }
}

void print_value(std::string const&indent, std::ostream &stream, value_accessor const&to_print) {
    stream << indent << "Label: " << to_print.get_debug_label() << std::endl;
    stream << indent << "Type: " << to_print.get_type() << " (";
    to_print.get_type()->print_repr(stream);
    stream << ")" << std::endl;
    if (to_print.is_value_known()) {
        stream << indent << "Value: ";
        to_print.get_type()->format(stream, to_print.get_data());
        stream << std::endl;
    }
    if (to_print.get_root_value()->is_proxy()) {
        stream << indent << "Proxy for: ";
        stream << &to_print.get_root_value()->get_real_value() << std::endl;
    }
}

////////////////////////////////////////////////////////////////////////////////
// Com::command
////////////////////////////////////////////////////////////////////////////////
abstract_command::abstract_command() { }

abstract_command::abstract_command(augmentation_ptr aug):
    aug{aug} { }

abstract_command::arg_list const&abstract_command::get_inputs() const {
    return ins;
}

void abstract_command::add_input(arg_ptr input) {
    ins.push_back(input);
}

void abstract_command::clear_inputs() {
    ins.clear();
}

abstract_command::arg_list const&abstract_command::get_outputs() const {
    return outs;
}

void abstract_command::add_output(arg_ptr output) {
    outs.push_back(output);
}

void abstract_command::clear_outputs() {
    outs.clear();
}

std::vector<command_lambda> const&abstract_command::get_lambdas() const {
    return lambdas;
}

void abstract_command::add_lambda(command_lambda &lambda) {
    lambdas.push_back(lambda);
}

const augmentation_ptr abstract_command::get_augmentation() const {
    return aug;
}

command::command() { }

command::command(scope_ptr callee)
    : callee(callee) { }

command::command(scope_ptr callee, augmentation_ptr aug)
    : abstract_command(aug), callee(callee) { }

const scope_ptr command::get_callee() const {
    return callee;
}

void command::set_callee(scope_ptr callee) {
    callee = callee;
}

resolved_command::resolved_command() { }

resolved_command::resolved_command(resolved_scope_ptr callee)
    : callee(callee) { }

resolved_command::resolved_command(resolved_scope_ptr callee, 
    augmentation_ptr aug)
    : abstract_command(aug), callee(callee) { }

const resolved_scope_ptr resolved_command::get_callee() const {
    return callee;
}

void resolved_command::set_callee(resolved_scope_ptr callee) {
    callee = callee;
}

std::ostream &operator<<(std::ostream &stream, abstract_command const&to_print) {
    for (auto value : to_print.ins) {
        stream << "      Input: " << value->get_root_value() << std::endl;
        print_value("        ", stream, *value);
    }
    for (auto value : to_print.outs) {
        stream << "      Output: " << value->get_root_value() << std::endl;
        print_value("        ", stream, *value);
    }
    for (auto lambda : to_print.lambdas) {
        stream << "      Lambda: " << lambda.body << " is " << lambda.name
            << std::endl;
    }
    return stream;
}

std::ostream &operator<<(std::ostream &stream, command const&to_print) {
    stream << "    " << to_print.callee->get_debug_path() << std::endl;
    stream << static_cast<abstract_command const&>(to_print);
    return stream;
}

std::ostream &operator<<(std::ostream &stream,  
    resolved_command const&to_print) {
    stream << "    " << to_print.callee->get_debug_path() << std::endl;
    stream << static_cast<abstract_command const&>(to_print);
    return stream;
}

////////////////////////////////////////////////////////////////////////////////
// Com::scope
////////////////////////////////////////////////////////////////////////////////

scope::scope()
    : debug_label("unlabeled") { }

scope::scope(scope_ptr parent) 
    : debug_label("unlabeled"), parent(parent) { }

void scope::set_debug_label(std::string debug_label) {
    this->debug_label = debug_label;
}

const std::string scope::get_debug_label() const {
    return debug_label;
}

const std::string scope::get_debug_path() const {
    return (parent ? parent->get_debug_path() + "." : "") + debug_label;
}

const scope_ptr scope::get_parent() const {
    return parent;
}

std::ostream &operator<<(std::ostream &stream, scope const&to_print) {
    stream << to_print.get_debug_path() << " is Scope:" << std::endl;
    stream << "  Parent: " << to_print.parent.get() << std::endl;
    for (auto in : to_print.ins) {
    stream << "  Input: " << in->get_root_value() << std::endl;
        print_value("      ", stream, *in);
    }
    for (auto out : to_print.outs) {
        stream << "  Outputs: " << out->get_root_value() << std::endl;
        print_value("      ", stream, *out);
    }
    stream << "  Types:" << std::endl;
    for (auto type : to_print.types) {
        stream << "    " << type.second << " is " << type.first << std::endl;
    }
    stream << "  Function Declarations:" << std::endl;
    for (unsigned int i = 0; i < to_print.temp_funcs.size(); i++) {
        stream << "    " << to_print.temp_funcs[i]->get_debug_path() 
            << " is !TEMP" << (i + 1) << std::endl;
    }
    for (auto func : to_print.funcs) {
        stream << "    " << func.second->get_debug_path() << " is " 
            << func.first << std::endl;
    }
    stream << "  Variable Declarations:" << std::endl;
    for (unsigned int i = 0; i < to_print.temp_vars.size(); i++) {
        stream << "    " << to_print.temp_vars[i] << " is !TEMP" << (i + 1) 
            << ":" << std::endl;
        print_value("      ", stream, *to_print.temp_vars[i]);
    }
    for (auto var : to_print.vars) {
        stream << "    " << var.second << " is " << var.first << ":" 
            << std::endl;
        print_value("      ", stream, *var.second);
    }
    stream << "  Commands:" << std::endl;
    for (auto command : to_print.commands) {
        stream << *command;
    }
    for (auto child : to_print.temp_funcs) {
        stream << std::endl << *child;
    }
    for (auto child : to_print.funcs) {
        stream << std::endl << *child.second;
    }
    return stream;
}

void scope::declare_func(std::string name, scope_ptr body) {
    funcs.emplace(name, body);
    body->set_debug_label(name);
}

void scope::declare_temp_func(scope_ptr body) {
    temp_funcs.push_back(body);
    body->set_debug_label("!TEMP" + std::to_string(temp_funcs.size()));
}

const scope_ptr scope::lookup_func(std::string name, bool recurse) 
    const {
    if (funcs.count(name)) {
        return funcs.at(name);
    } else if (parent && recurse) {
        return parent->lookup_func(name);
    } else {
        return nullptr;
    }
}

const std::map<std::string, scope_ptr> scope::get_func_table() const {
    return funcs;
}

const std::vector<scope_ptr> scope::get_temp_func_list() const {
    return temp_funcs;
}

void scope::declare_var(std::string name, value_ptr value) {
    vars.emplace(name, value);
    value->set_debug_label("Variable " + name);
}

void scope::declare_temp_var(value_ptr value) {
    temp_vars.push_back(value);
    value->set_debug_label("Temp Var #" + std::to_string(temp_vars.size()));
}

const value_ptr scope::lookup_var(std::string name, bool recurse) 
    const {
    if (vars.count(name)) {
        return vars.at(name);
    } else if (parent && recurse) {
        return parent->lookup_var(name);
    } else {
        return nullptr;
    }
}

const std::map<std::string, value_ptr> scope::get_var_table() const {
    return vars;
}

const std::vector<value_ptr> scope::get_temp_var_list() const {
    return temp_vars;
}

void scope::declare_type(std::string name, data_type_ptr type) {
    types.emplace(name, type);
}

const data_type_ptr scope::lookup_type(std::string name, 
    bool recurse) const {
    if (types.count(name)) {
        return types.at(name);
    } else if (parent && recurse) {
        return parent->lookup_type(name);
    } else {
        return nullptr;
    }
}

const std::map<std::string, data_type_ptr> scope::get_type_table() const {
    return types;
}

void scope::add_command(command_ptr command) {
    commands.push_back(command);
}

const std::vector<command_ptr> &scope::get_commands() const {
    return commands;
}

void scope::clear_commands() {
    commands.clear();
}

value_ptr scope::add_input(std::string name, vague_data_type_ptr type) {
    auto value_type{std::make_shared<unresolved_vague_type>(type)};
    auto holder{std::make_shared<value>(value_type)};
    ins.push_back(std::make_shared<value_accessor>(holder));
    // TODO: Expose types and values used in the template for the body of the
    // scope to use.
    // TODO: mark inputs as read-only.
    declare_var(name, holder);
    return holder;
}

void scope::add_resolved_input(const_value_accessor_ptr input) {
    ins.push_back(input);
}

const std::vector<const_value_accessor_ptr> &scope::get_inputs() const {
    return ins;
}

value_ptr scope::add_output(std::string name, vague_data_type_ptr type) {
    auto value_type{std::make_shared<unresolved_vague_type>(type)};
    auto holder{std::make_shared<value>(value_type)};
    outs.push_back(std::make_shared<value_accessor>(holder));
    // TODO: Expose types and values used in the template for the body of the
    // scope to use.
    declare_var(name, holder);
    return holder;
}

void scope::add_resolved_output(const_value_accessor_ptr output) {
    outs.push_back(output);
}

const std::vector<const_value_accessor_ptr> &scope::get_outputs() const {
    return outs;
}



resolved_scope::resolved_scope() { }

resolved_scope::resolved_scope(resolved_scope_ptr parent)
    : parent(parent) { }

void resolved_scope::set_debug_label(std::string debug_label) {
    this->debug_label = debug_label;
}

const std::string resolved_scope::get_debug_label() const {
    return debug_label;
}

const std::string resolved_scope::get_debug_path() const {
    if (parent) {
        return parent->get_debug_path() + "." + debug_label;
    } else {
        return debug_label;
    }
}

void resolved_scope::add_command(resolved_command_ptr command) {
    commands.push_back(command);
}

void resolved_scope::clear_commands() {
    commands.clear();
}

std::vector<resolved_command_ptr> const&resolved_scope::get_commands() const {
    return commands;
}

void resolved_scope::add_value_conversion(const_value_ptr from, value_ptr to) {
    value_conversions[from.get()] = to;
}

value_map const&resolved_scope::get_value_conversions() const {
    return value_conversions;
}

const_value_ptr resolved_scope::convert_value(const_value_ptr from) 
    const {
    if (value_conversions.count(from.get())) {
        return value_conversions.at(from.get());
    } else if (parent) {
        return parent->convert_value(from);
    } else {
        return from;
    }
}

const_value_accessor_ptr resolved_scope::convert_value(
    const_value_accessor_ptr from) const {
    if (value_conversions.count(from->get_root_value().get())) {
        auto new_ptr{std::make_shared<value_accessor>()};
        new_ptr->set_root_value(value_conversions.at(
            from->get_root_value().get()
        ));
        for (auto subpart : from->get_subparts()) {
            new_ptr->add_subpart(subpart);
        }
        return new_ptr;
    } else if (parent) {
        return parent->convert_value(from);
    } else {
        return from;
    }
}

void resolved_scope::add_data_type_conversion(const_data_type_ptr from, 
    const_data_type_ptr to) {
    data_type_conversions[from.get()] = to;
}

data_type_map const&resolved_scope::get_data_type_conversions() const {
    return data_type_conversions;
}

const_data_type_ptr resolved_scope::convert_data_type(const_data_type_ptr from) 
    const {
    if (data_type_conversions.count(from.get())) {
        return data_type_conversions.at(from.get());
    } else if (parent) {
        return parent->convert_data_type(from);
    } else {
        return from;
    }
}

void resolved_scope::add_resolved_input(const_value_accessor_ptr input) {
    ins.push_back(input);
}

std::vector<const_value_accessor_ptr> const&resolved_scope::get_inputs() const {
    return ins;
}

void resolved_scope::add_resolved_output(const_value_accessor_ptr output) {
    outs.push_back(output);
}

std::vector<const_value_accessor_ptr> const&resolved_scope::get_outputs() const {
    return outs;
}

std::ostream &operator<<(std::ostream &stream, resolved_scope const&to_print) {
    stream << to_print.get_debug_path() << " is Resolved Scope:" << std::endl;
    stream << "  Parent: " << to_print.parent.get() << std::endl;
    for (auto in : to_print.ins) {
        stream << "  Inputs: " << in << std::endl;
        print_value("    ", stream, *in);
    }
    for (auto out : to_print.outs) {
        stream << "  Outputs: " << out << std::endl;
        print_value("    ", stream, *out);
    }
    stream << "  Commands:" << std::endl;
    for (auto command : to_print.commands) {
        stream << *command;
    }

    std::set<resolved_scope_ptr> child_scopes;
    for (auto command : to_print.commands) {
        child_scopes.insert(command->get_callee());
    }
    for (auto callee : child_scopes) {
        stream << std::endl << *callee;
    }
    return stream;
}

}
}