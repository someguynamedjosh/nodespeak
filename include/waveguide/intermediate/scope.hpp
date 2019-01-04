#pragma once

#include <boost/variant.hpp>
#include <map>
#include <memory>
#include <ostream>
#include <string>
#include <vector>

namespace waveguide {
namespace intermediate {

class augmentation;
class command;
class data_type;
class scope;
class vague_data_type;
class value;

typedef std::shared_ptr<augmentation> augmentation_ptr;
typedef std::shared_ptr<command> command_ptr;
typedef std::shared_ptr<data_type> data_type_ptr;
typedef std::shared_ptr<scope> scope_ptr;
typedef std::shared_ptr<vague_data_type> vague_data_type_ptr;
typedef std::shared_ptr<value> value_ptr;

struct do_if_aug {
    value_ptr condition;
    friend std::ostream &operator<<(std::ostream &stream, 
        do_if_aug const&to_print);
};

struct do_if_not_aug {
    value_ptr condition;
    friend std::ostream &operator<<(std::ostream &stream, 
        do_if_not_aug const&to_print);
};

struct loop_for_aug {
    value_ptr to_set, iterate_over;
    friend std::ostream &operator<<(std::ostream &stream, 
        loop_for_aug const&to_print);
};

struct loop_range_aug {
    value_ptr to_set, start, end, step;
    friend std::ostream &operator<<(std::ostream &stream, 
        loop_range_aug const&to_print);
};

std::ostream &operator<<(std::ostream &stream, do_if_aug const&to_print);
std::ostream &operator<<(std::ostream &stream, do_if_not_aug const&to_print);
std::ostream &operator<<(std::ostream &stream, loop_for_aug const&to_print);
std::ostream &operator<<(std::ostream &stream, loop_range_aug const&to_print);

struct augmentation: boost::variant<
    do_if_aug, do_if_not_aug, loop_for_aug, loop_range_aug
> { };

struct command_lambda {
    std::string name;
    scope_ptr body;
};

class command {
private:
    scope_ptr call{nullptr};
    std::vector<value_ptr> ins, outs;
    std::vector<command_lambda> lambdas;
    augmentation_ptr aug{nullptr};
public:
    command(scope_ptr call);
    command(scope_ptr call, augmentation_ptr aug);
    friend std::ostream &operator<<(std::ostream &stream, 
        command const&to_print);

    void add_input(value_ptr input);
    void clear_inputs();
    void add_output(value_ptr output);
    void clear_outputs();
    void add_lambda(command_lambda &lambda);
    std::vector<value_ptr> const&get_inputs() const;
    std::vector<value_ptr> const&get_outputs() const;
    std::vector<command_lambda> const&get_lambdas() const;
    const augmentation_ptr get_augmentation() const;
    const scope_ptr get_called_scope() const;
};
std::ostream &operator<<(std::ostream &stream, command const&to_print);

class scope {
private:
    scope_ptr parent{nullptr};
    std::map<std::string, scope_ptr> funcs;
    std::vector<scope_ptr> temp_funcs;
    std::map<std::string, value_ptr> vars;
    std::vector<value_ptr> temp_vars;
    std::map<std::string, data_type_ptr> types;
    std::vector<command_ptr> commands;
    std::vector<value_ptr> ins, outs;
public:
    scope();
    scope(scope_ptr parent);
    const scope_ptr get_parent() const;
    friend std::ostream &operator<<(std::ostream &stream, scope const&to_print);

    void declare_func(std::string name, scope_ptr body);
    void declare_temp_func(scope_ptr body);
    const scope_ptr lookup_func(std::string name, bool recurse = true) const;
    void declare_var(std::string name, value_ptr value);
    void declare_temp_var(value_ptr value);
    const value_ptr lookup_var(std::string name, bool recurse = true) const;
    void declare_type(std::string name, data_type_ptr type);
    const data_type_ptr lookup_type(std::string name, bool recurse = true) 
        const;
    void add_command(command_ptr command);
    void clear_commands();
    const std::vector<command_ptr> &get_commands() const;

    value_ptr add_input(std::string name, vague_data_type_ptr type);
    const std::vector<value_ptr> &get_inputs() const;
    value_ptr add_output(std::string name, vague_data_type_ptr type);
    const std::vector<value_ptr> &get_outputs() const;
};
std::ostream &operator<<(std::ostream &stream, scope const&to_print);

}
}
