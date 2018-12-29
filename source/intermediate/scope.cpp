#include <waveguide/intermediate/data_type.hpp>
#include <waveguide/intermediate/scope.hpp>
#include <waveguide/intermediate/value.hpp>
#include <sstream>

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

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

////////////////////////////////////////////////////////////////////////////////
// Com::command
////////////////////////////////////////////////////////////////////////////////
command::command(std::shared_ptr<scope> call)
    : call{call} { }

command::command(std::shared_ptr<scope> call, std::shared_ptr<augmentation> aug)
    : call{call}, aug{aug} { }

std::ostream &operator<<(std::ostream &stream, command const&to_print) {
    stream << "    " << to_print.call << std::endl;
    for (auto value : to_print.ins) {
        stream << "      Input: " << value << " (type " << 
            value->get_type()->repr() << ")";
        if (value->is_value_known()) {
            stream << " = " << value->get_type()->format(value->get_data());
        }
        stream << std::endl;
    }
    for (auto value : to_print.outs) {
        stream << "      Output: " << value << " (type " << 
            value->get_type()->repr() << ")";
        if (value->is_value_known()) {
            stream << " = " << value->get_type()->format(value->get_data());
        }
        stream << std::endl;
    }
    for (auto lambda : to_print.lambdas) {
        stream << "      Lambda: " << lambda.body << " is " << lambda.name
            << std::endl;
    }
    return stream;
}

void command::add_input(std::shared_ptr<value> input) {
    ins.push_back(input);
}

void command::add_output(std::shared_ptr<value> output) {
    outs.push_back(output);
}

void command::add_lambda(command_lambda &lambda) {
    lambdas.push_back(lambda);
}

std::vector<std::shared_ptr<value>> const&command::get_inputs() const {
    return ins;
}

std::vector<std::shared_ptr<value>> const&command::get_outputs() const {
    return outs;
}

std::vector<command_lambda> const&command::get_lambdas() const {
    return lambdas;
}

const std::shared_ptr<augmentation> command::get_augmentation() const {
    return aug;
}

const std::shared_ptr<scope> command::get_called_scope() const {
    return call;
}

////////////////////////////////////////////////////////////////////////////////
// Com::scope
////////////////////////////////////////////////////////////////////////////////

scope::scope() { }

scope::scope(std::shared_ptr<scope> parent) 
    : parent(parent) { }

const std::shared_ptr<scope> scope::get_parent() const {
    return parent;
}

void print_value(std::ostream &stream, value const&to_print) {
    stream << "      Type: " << to_print.get_type() << " (" << 
        to_print.get_type()->repr() << ")" << std::endl;
    if (to_print.is_value_known()) {
        stream << "      Value: " << 
            to_print.get_type()->format(to_print.get_data()) << std::endl;
    }
}

std::ostream &operator<<(std::ostream &stream, scope const&to_print) {
    stream << &to_print << " is Scope:" << std::endl;
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
        stream << "    " << to_print.temp_funcs[i] << " is !TEMP" << (i + 1) 
            << std::endl;
    }
    for (auto func : to_print.funcs) {
        stream << "    " << func.second << " is " << func.first << std::endl;
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

void scope::declare_func(std::string name, std::shared_ptr<scope> body) {
    funcs.emplace(name, body);
}

void scope::declare_temp_func(std::shared_ptr<scope> body) {
    temp_funcs.push_back(body);
}

const std::shared_ptr<scope> scope::lookup_func(std::string name) const {
    if (funcs.count(name)) {
        return funcs.at(name);
    } else if (parent) {
        return parent->lookup_func(name);
    } else {
        return nullptr;
    }
}

void scope::declare_var(std::string name, std::shared_ptr<value> value) {
    vars.emplace(name, value);
}

void scope::declare_temp_var(std::shared_ptr<value> value) {
    temp_vars.push_back(value);
}

const std::shared_ptr<value> scope::lookup_var(std::string name) const {
    if (vars.count(name)) {
        return vars.at(name);
    } else if (parent) {
        return parent->lookup_var(name);
    } else {
        return nullptr;
    }
}

void scope::declare_type(std::string name, std::shared_ptr<data_type> type) {
    types.emplace(name, type);
}

const std::shared_ptr<data_type> scope::lookup_type(std::string name) const {
    if (types.count(name)) {
        return types.at(name);
    } else if (parent) {
        return parent->lookup_type(name);
    } else {
        return nullptr;
    }
}

void scope::add_command(std::shared_ptr<command> command) {
    commands.push_back(command);
}

const std::vector<std::shared_ptr<command>> &scope::get_commands() const {
    return commands;
}

SP<value> scope::add_input(std::string name, SP<vague_data_type> type) {
    SP<data_type> value_type{new unresolved_vague_type{type}};
    SP<value> holder{new value{value_type}};
    ins.push_back(holder);
    // TODO: Expose types and values used in the template for the body of the
    // scope to use.
    // TODO: mark inputs as read-only.
    declare_var(name, holder);
    return holder;
}

const std::vector<std::shared_ptr<value>> &scope::get_inputs() const {
    return ins;
}

SP<value> scope::add_output(std::string name, SP<vague_data_type> type) {
    SP<data_type> value_type{new unresolved_vague_type{type}};
    SP<value> holder{new value{value_type}};
    outs.push_back(holder);
    // TODO: Expose types and values used in the template for the body of the
    // scope to use.
    declare_var(name, holder);
    return holder;
}

const std::vector<std::shared_ptr<value>> &scope::get_outputs() const {
    return outs;
}

}
}