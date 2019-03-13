#pragma once

#include <boost/variant.hpp>
#include <map>
#include <memory>
#include <ostream>
#include <string>
#include <vector>

namespace waveguide {
namespace intermediate {

class abstract_command;
class command;
class resolved_command;
class data_type;
class scope;
class resolved_scope;
class vague_data_type;
class value;
class value_accessor;

typedef std::shared_ptr<command> command_ptr;
typedef std::shared_ptr<resolved_command> resolved_command_ptr;
typedef std::shared_ptr<data_type> data_type_ptr;
typedef std::shared_ptr<const data_type> const_data_type_ptr;
typedef std::shared_ptr<scope> scope_ptr;
typedef std::shared_ptr<resolved_scope> resolved_scope_ptr;
typedef std::shared_ptr<vague_data_type> vague_data_type_ptr;
typedef std::shared_ptr<value> value_ptr;
typedef std::shared_ptr<value_accessor> value_accessor_ptr;
typedef std::shared_ptr<const value> const_value_ptr;
typedef std::shared_ptr<const value_accessor> const_value_accessor_ptr;
typedef std::map<const value*, const_value_ptr> value_map;
typedef std::map<const data_type*, const_data_type_ptr> data_type_map;

struct do_if_aug {
    value_accessor_ptr condition;
    friend std::ostream &operator<<(std::ostream &stream, 
        do_if_aug const&to_print);
};

struct do_if_not_aug {
    value_accessor_ptr condition;
    friend std::ostream &operator<<(std::ostream &stream, 
        do_if_not_aug const&to_print);
};

struct loop_for_aug {
    value_accessor_ptr to_set, iterate_over;
    friend std::ostream &operator<<(std::ostream &stream, 
        loop_for_aug const&to_print);
};

struct loop_range_aug {
    value_accessor_ptr to_set, start, end, step;
    friend std::ostream &operator<<(std::ostream &stream, 
        loop_range_aug const&to_print);
};

std::ostream &operator<<(std::ostream &stream, do_if_aug const&to_print);
std::ostream &operator<<(std::ostream &stream, do_if_not_aug const&to_print);
std::ostream &operator<<(std::ostream &stream, loop_for_aug const&to_print);
std::ostream &operator<<(std::ostream &stream, loop_range_aug const&to_print);

typedef boost::variant<do_if_aug, do_if_not_aug, loop_for_aug, loop_range_aug>
    augmentation;
typedef std::shared_ptr<augmentation> augmentation_ptr;

struct command_lambda {
    std::string name;
    scope_ptr body;
};

class abstract_command {
private:
    typedef const_value_accessor_ptr arg_ptr;
    typedef std::vector<arg_ptr> arg_list;
    arg_list ins, outs;
    std::vector<command_lambda> lambdas;
    augmentation_ptr aug{nullptr};
public:
    abstract_command();
    abstract_command(augmentation_ptr aug);
    friend std::ostream &operator<<(std::ostream &stream, 
        abstract_command const&to_print);

    arg_list const&get_inputs() const;
    void add_input(arg_ptr input);
    void clear_inputs();

    arg_list const&get_outputs() const;
    void add_output(arg_ptr output);
    void clear_outputs();

    std::vector<command_lambda> const&get_lambdas() const;
    void add_lambda(command_lambda &lambda);
    void clear_lambdas();

    const augmentation_ptr get_augmentation() const;
};
std::ostream &operator<<(std::ostream &stream, abstract_command const&to_print);

class command: public abstract_command {
private:
    scope_ptr callee{nullptr};
public:
    command();
    command(scope_ptr callee);
    command(scope_ptr callee, augmentation_ptr aug);
    friend std::ostream &operator<<(std::ostream &stream, 
        command const&to_print);

    const scope_ptr get_callee() const;
    void set_callee(scope_ptr callee);
};
std::ostream &operator<<(std::ostream &stream, command const&to_print);

class resolved_command: public abstract_command {
private:
    resolved_scope_ptr callee{nullptr};
public:
    resolved_command();
    resolved_command(resolved_scope_ptr callee);
    resolved_command(resolved_scope_ptr callee, augmentation_ptr aug);
    friend std::ostream &operator<<(std::ostream &stream, 
        resolved_command const&to_print);

    const resolved_scope_ptr get_callee() const;
    void set_callee(resolved_scope_ptr callee);
};
std::ostream &operator<<(std::ostream &stream, resolved_command const&to_print);

class scope {
private:
    // TODO: Remove this value in production builds.
    std::string debug_label;
    scope_ptr parent{nullptr};
    std::map<std::string, scope_ptr> funcs;
    std::vector<scope_ptr> temp_funcs;
    std::map<std::string, value_ptr> vars;
    std::vector<value_ptr> temp_vars;
    std::map<std::string, data_type_ptr> types;
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

    void declare_type(std::string name, data_type_ptr type);
    const data_type_ptr lookup_type(std::string name, bool recurse = true) 
        const;
    const std::map<std::string, data_type_ptr> get_type_table() const;

    void add_command(command_ptr command);
    void clear_commands();
    const std::vector<command_ptr> &get_commands() const;

    value_ptr add_input(std::string name, vague_data_type_ptr type);
    void add_resolved_input(const_value_accessor_ptr input);
    const std::vector<const_value_accessor_ptr> &get_inputs() const;
    value_ptr add_output(std::string name, vague_data_type_ptr type);
    void add_resolved_output(const_value_accessor_ptr output);
    const std::vector<const_value_accessor_ptr> &get_outputs() const;
};
std::ostream &operator<<(std::ostream &stream, scope const&to_print);

class resolved_scope {
private:
    // TODO: Remove these two value in production builds.
    std::string debug_label;
    resolved_scope_ptr parent{nullptr};
    value_map value_conversions;
    data_type_map data_type_conversions;
    std::vector<resolved_command_ptr> commands;
    std::vector<const_value_accessor_ptr> ins, outs;
public:
    resolved_scope();
    resolved_scope(resolved_scope_ptr parent);
    void set_debug_label(std::string debug_label);
    const std::string get_debug_label() const;
    const std::string get_debug_path() const;
    friend std::ostream &operator<<(std::ostream &stream, 
        resolved_scope const&to_print);

    void add_command(resolved_command_ptr command);
    void clear_commands();
    std::vector<resolved_command_ptr> const&get_commands() const;

    void add_value_conversion(const_value_ptr from, const_value_ptr to);
    value_map const&get_value_conversions() const;
    const_value_ptr convert_value(const_value_ptr from) const;
    const_value_accessor_ptr convert_value(const_value_accessor_ptr from) const;
    void add_data_type_conversion(const_data_type_ptr from, 
        const_data_type_ptr to);
    data_type_map const&get_data_type_conversions() const;
    const_data_type_ptr convert_data_type(const_data_type_ptr from) const;

    void add_resolved_input(const_value_accessor_ptr input);
    std::vector<const_value_accessor_ptr> const&get_inputs() const;
    void add_resolved_output(const_value_accessor_ptr output);
    std::vector<const_value_accessor_ptr> const&get_outputs() const;
};
std::ostream &operator<<(std::ostream &stream, resolved_scope const&to_print);

}
}
