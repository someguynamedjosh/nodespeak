#include <waveguide/intermediate/data_type.hpp>
#include <waveguide/intermediate/scope.hpp>
#include <waveguide/intermediate/value.hpp>
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

////////////////////////////////////////////////////////////////////////////////
// Com::command
////////////////////////////////////////////////////////////////////////////////
abstract_command::abstract_command() { }

abstract_command::abstract_command(augmentation_ptr aug):
    aug{aug} { }

std::vector<value_ptr> const&abstract_command::get_inputs() const {
    return ins;
}

void abstract_command::add_input(value_ptr input) {
    ins.push_back(input);
}

void abstract_command::clear_inputs() {
    ins.clear();
}

std::vector<value_ptr> const&abstract_command::get_outputs() const {
    return outs;
}

void abstract_command::add_output(value_ptr output) {
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

const scope_ptr command::get_callee() const {
    return callee;
}

void command::set_callee(scope_ptr callee) {
    callee = callee;
}

const resolved_scope_ptr resolved_command::get_callee() const {
    return callee;
}

void resolved_command::set_callee(resolved_scope_ptr callee) {
    callee = callee;
}

std::ostream &operator<<(std::ostream &stream, abstract_command const&to_print) {
    for (auto value : to_print.ins) {
        stream << "      Input: " << value << " (type ";
        value->get_type()->print_repr(stream);
        stream << ")";
        if (value->is_value_known()) {
            stream << " = ";
            value->get_type()->format(stream, value->get_data().get());
        }
        stream << std::endl;
    }
    for (auto value : to_print.outs) {
        stream << "      Output: " << value << " (type ";
        value->get_type()->print_repr(stream);
        stream << ")";
        if (value->is_value_known()) {
            stream << " = ";
            value->get_type()->format(stream, value->get_data().get());
        }
        stream << std::endl;
    }
    for (auto lambda : to_print.lambdas) {
        stream << "      Lambda: " << lambda.body << " is " << lambda.name
            << std::endl;
    }
    return stream;
}
    stream << "    " << to_print.callee->get_debug_path() << std::endl;

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

void print_value(std::ostream &stream, value const&to_print) {
    stream << "      Type: " << to_print.get_type() << " (";
    to_print.get_type()->print_repr(stream);
    stream << ")" << std::endl;
    if (to_print.is_value_known()) {
        stream << "      Value: ";
        to_print.get_type()->format(stream, to_print.get_data().get());
        stream << std::endl;
    }
    if (to_print.is_proxy()) {
        stream << "      Proxy for: ";
        stream << &to_print.get_real_value() << std::endl;
    }
}

std::ostream &operator<<(std::ostream &stream, scope const&to_print) {
    stream << to_print.get_debug_path() << " is Scope:" << std::endl;
    stream << "  Parent: " << to_print.parent.get() << std::endl;
    stream << "  Inputs:" << std::endl;
    for (auto in : to_print.ins) {
        print_value(stream, *in);
    }
    stream << "  Outputs:" << std::endl;
    for (auto out : to_print.outs) {
        print_value(stream, *out);
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
        print_value(stream, *to_print.temp_vars[i]);
    }
    for (auto var : to_print.vars) {
        stream << "    " << var.second << " is " << var.first << ":" 
            << std::endl;
        print_value(stream, *var.second);
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
}

void scope::declare_temp_var(value_ptr value) {
    temp_vars.push_back(value);
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
    ins.push_back(holder);
    // TODO: Expose types and values used in the template for the body of the
    // scope to use.
    // TODO: mark inputs as read-only.
    declare_var(name, holder);
    return holder;
}

void scope::add_resolved_input(value_ptr input) {
    ins.push_back(input);
}

const std::vector<value_ptr> &scope::get_inputs() const {
    return ins;
}

value_ptr scope::add_output(std::string name, vague_data_type_ptr type) {
    auto value_type{std::make_shared<unresolved_vague_type>(type)};
    auto holder{std::make_shared<value>(value_type)};
    outs.push_back(holder);
    // TODO: Expose types and values used in the template for the body of the
    // scope to use.
    declare_var(name, holder);
    return holder;
}

void scope::add_resolved_output(value_ptr output) {
    outs.push_back(output);
}

const std::vector<value_ptr> &scope::get_outputs() const {
    return outs;
}

}
}