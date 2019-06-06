#pragma once

#include <map>
#include <memory>
#include <string>
#include <vector>

namespace waveguide {

namespace vague {

class command;
class scope;
class template_data_type;
class value;
class value_accessor;

typedef std::shared_ptr<command> command_ptr;
typedef std::shared_ptr<scope> scope_ptr;
typedef std::shared_ptr<template_data_type> template_data_type_ptr;
typedef std::shared_ptr<value> value_ptr;
typedef std::shared_ptr<value_accessor> value_accessor_ptr;
typedef std::shared_ptr<const value_accessor> const_value_accessor_ptr;

class scope {
private:
    // TODO: Remove this value in production builds.
    std::string debug_label;
    scope_ptr parent{nullptr};
    std::map<std::string, scope_ptr> funcs;
    std::vector<scope_ptr> temp_funcs;
    std::map<std::string, value_ptr> vars;
    std::vector<value_ptr> temp_vars;
    std::map<std::string, template_data_type_ptr> types;
    std::vector<command_ptr> commands;
    std::vector<const_value_accessor_ptr> ins, outs;
public:
    scope();
    scope(scope_ptr parent);
    void set_debug_label(std::string debug_label);
    const std::string get_debug_label() const;
    const std::string get_debug_path() const;
    const scope_ptr get_parent() const;
    friend std::ostream &operator<<(std::ostream &stream, scope const&to_print);

    void declare_func(std::string name, scope_ptr body);
    void declare_temp_func(scope_ptr body);
    const scope_ptr lookup_func(std::string name, bool recurse = true) const;
    const std::map<std::string, scope_ptr> get_func_table() const;
    const std::vector<scope_ptr> get_temp_func_list() const;

    void declare_var(std::string name, value_ptr value);
    void declare_temp_var(value_ptr value);
    const value_ptr lookup_var(std::string name, bool recurse = true) const;
    const std::map<std::string, value_ptr> get_var_table() const;
    const std::vector<value_ptr> get_temp_var_list() const;

    void declare_type(std::string name, template_data_type_ptr type);
    const template_data_type_ptr lookup_type(std::string name, 
        bool recurse = true) const;
    const std::map<std::string, template_data_type_ptr> get_type_table() const;

    void add_command(command_ptr command);
    void clear_commands();
    const std::vector<command_ptr> &get_commands() const;

    value_ptr add_input(std::string name, template_data_type_ptr type);
    void add_resolved_input(const_value_accessor_ptr input);
    const std::vector<const_value_accessor_ptr> &get_inputs() const;
    value_ptr add_output(std::string name, template_data_type_ptr type);
    void add_resolved_output(const_value_accessor_ptr output);
    const std::vector<const_value_accessor_ptr> &get_outputs() const;
};
std::ostream &operator<<(std::ostream &stream, scope const&to_print);


}
}