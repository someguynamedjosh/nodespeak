#pragma once

#include <boost/variant.hpp>
#include <map>
#include <memory>
#include <ostream>
#include <string>
#include <vector>

namespace waveguide {
namespace intermediate {

class data_type;
class scope;
class value;

struct do_if_aug {
    std::shared_ptr<value> condition;
    friend std::ostream &operator<<(std::ostream &stream, 
        do_if_aug const&to_print);
};

struct do_if_not_aug {
    std::shared_ptr<value> condition;
    friend std::ostream &operator<<(std::ostream &stream, 
        do_if_not_aug const&to_print);
};

struct loop_for_aug {
    std::shared_ptr<value> to_set, iterate_over;
    friend std::ostream &operator<<(std::ostream &stream, 
        loop_for_aug const&to_print);
};

struct loop_range_aug {
    std::shared_ptr<value> to_set, start, end, step;
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
    std::shared_ptr<scope> body;
};

class command {
    private:
    std::shared_ptr<scope> call{nullptr};
    std::vector<std::shared_ptr<value>> ins, outs;
    std::vector<command_lambda> lambdas;
    std::shared_ptr<augmentation> aug{nullptr};
public:
    command(std::shared_ptr<scope> call);
    command(std::shared_ptr<scope> call, std::shared_ptr<augmentation> aug);
    friend std::ostream &operator<<(std::ostream &stream, 
        command const&to_print);

    void add_input(std::shared_ptr<value> input);
    void add_output(std::shared_ptr<value> output);
    void add_lambda(command_lambda &lambda);
    std::vector<std::shared_ptr<value>> const&get_inputs() const;
    std::vector<std::shared_ptr<value>> const&get_outputs() const;
    std::vector<command_lambda> const&get_lambdas() const;
    const std::shared_ptr<augmentation> get_augmentation() const;
    const std::shared_ptr<scope> get_called_scope() const;
};
std::ostream &operator<<(std::ostream &stream, command const&to_print);

class scope {
private:
    std::map<std::string, std::shared_ptr<scope>> funcs;
    std::vector<std::shared_ptr<scope>> temp_funcs;
    std::map<std::string, std::shared_ptr<value>> vars;
    std::vector<std::shared_ptr<value>> temp_vars;
    std::map<std::string, std::shared_ptr<data_type>> types;
    std::vector<std::shared_ptr<command>> commands;
    std::shared_ptr<scope> parent{nullptr};

    enum auto_add {
        NONE, INS, OUTS
    };
    auto_add do_auto{auto_add::NONE};
    std::vector<std::shared_ptr<value>> ins, outs;
public:
    scope();
    scope(std::shared_ptr<scope> parent);
    const std::shared_ptr<scope> get_parent() const;
    friend std::ostream &operator<<(std::ostream &stream, 
        scope const&to_print);

    void declare_func(std::string name, std::shared_ptr<scope> body);
    void declare_temp_func(std::shared_ptr<scope> body);
    const std::shared_ptr<scope> lookup_func(std::string name) const;
    void declare_var(std::string name, std::shared_ptr<value> value);
    void declare_temp_var(std::shared_ptr<value> value);
    const std::shared_ptr<value> lookup_var(std::string name) const;
    void declare_type(std::string name, std::shared_ptr<data_type> type);
    const std::shared_ptr<data_type> lookup_type(std::string name) const;
    void add_command(std::shared_ptr<command> command);
    const std::vector<std::shared_ptr<command>> &get_commands() const;

    void add_input(std::shared_ptr<value> in);
    const std::vector<std::shared_ptr<value>> &get_inputs() const;
    void add_output(std::shared_ptr<value> out);
    const std::vector<std::shared_ptr<value>> &get_outputs() const;

    void auto_add_none();
    void auto_add_inputs();
    void auto_add_outputs();
};
std::ostream &operator<<(std::ostream &stream, scope const&to_print);

}
}
