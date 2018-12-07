#pragma once

#include <boost/variant.hpp>
#include <map>
#include <memory>
#include <string>
#include <vector>

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

class data_type;
class scope;
class value;

struct do_if_aug {
    SP<value> condition;
};

struct do_if_not_aug {
    SP<value> condition;
};

struct loop_for_aug {
    SP<value> to_set, iterate_over;
};

struct loop_range_aug {
    SP<value> to_set, start, end, step;
};

struct augmentation: boost::variant<
    do_if_aug, do_if_not_aug, loop_for_aug, loop_range_aug
> { };

class command {
    private:
    std::vector<SP<value>> ins, outs;
    SP<scope> call{nullptr};
    SP<augmentation> aug{nullptr};
public:
    command(SP<scope> call);
    command(SP<scope> call, SP<augmentation> aug);
    std::string repr();

    void add_input(SP<value> input);
    void add_output(SP<value> output);
    const std::vector<SP<value>> &get_inputs() const;
    const std::vector<SP<value>> &get_outputs() const;
    const SP<augmentation> get_augmentation() const;
    const SP<scope> get_called_scope() const;
};

class scope {
private:
    std::map<std::string, SP<scope>> funcs;
    std::vector<SP<scope>> tempFuncs;
    std::map<std::string, SP<value>> vars;
    std::vector<SP<value>> tempVars;
    std::map<std::string, SP<data_type>> types;
    std::vector<SP<command>> commands;
    SP<scope> parent{nullptr};

    enum auto_add {
        NONE, INS, OUTS
    };
    auto_add do_auto{auto_add::NONE};
    std::vector<SP<value>> ins, outs;
public:
    scope();
    scope(SP<scope> parent);
    const SP<scope> get_parent() const;
    std::string repr();

    void declare_func(std::string name, SP<scope> body);
    void declare_temp_func(SP<scope> body);
    const SP<scope> lookup_func(std::string name) const;
    void declare_var(std::string name, SP<value> value);
    void declare_temp_var(SP<value> value);
    const SP<value> lookup_var(std::string name) const;
    void declare_type(std::string name, SP<data_type> type);
    const SP<data_type> lookup_type(std::string name) const;
    void add_command(SP<command> command);
    const std::vector<SP<command>> &get_commands() const;

    void add_input(SP<value> in);
    const std::vector<SP<value>> &get_inputs() const;
    void add_output(SP<value> out);
    const std::vector<SP<value>> &get_outputs() const;

    void auto_add_none();
    void auto_add_inputs();
    void auto_add_outputs();
};

}
}
