#pragma once

#include <boost/variant.hpp>
#include <map>
#include <memory>
#include <string>
#include <vector>

namespace waveguide {
namespace intermediate {

class DataType;
class Scope;
class Value;

template<typename T>
using SP = std::shared_ptr<T>;

struct do_if_aug {
    SP<Value> condition;
};

struct do_if_not_aug {
    SP<Value> condition;
};

struct loop_for_aug {
    SP<Value> to_set, iterate_over;
};

struct loop_range_aug {
    SP<Value> to_set, start, end, step;
};

struct augmentation: boost::variant<
    do_if_aug, do_if_not_aug, loop_for_aug, loop_range_aug
> { };

class Command {
    private:
    std::vector<SP<Value>> ins, outs;
    SP<Scope> call{nullptr};
    SP<augmentation> aug{nullptr};
public:
    Command(SP<Scope> call);
    Command(SP<Scope> call, SP<augmentation> aug);
    std::string repr();

    void add_input(SP<Value> input);
    void add_output(SP<Value> output);
    std::vector<SP<Value>> &get_inputs();
    std::vector<SP<Value>> &get_outputs();
    SP<augmentation> get_augmentation();
    SP<Scope> get_called_scope();
};

class Scope {
private:
    std::map<std::string, SP<Scope>> funcs;
    std::vector<SP<Scope>> tempFuncs;
    std::map<std::string, SP<Value>> vars;
    std::vector<SP<Value>> tempVars;
    std::map<std::string, SP<DataType>> types;
    std::vector<SP<Command>> commands;
    SP<Scope> parent{nullptr};

    enum AutoAdd {
        NONE, INS, OUTS
    };
    AutoAdd autoAdd{AutoAdd::NONE};
    std::vector<SP<Value>> ins, outs;
public:
    Scope();
    Scope(SP<Scope> parent);
    SP<Scope> getParent();
    std::string repr();

    void declare_func(std::string name, SP<Scope> body);
    void declare_temp_func(SP<Scope> body);
    SP<Scope> lookup_func(std::string name);
    void declare_var(std::string name, SP<Value> value);
    void declare_temp_var(SP<Value> value);
    SP<Value> lookup_var(std::string name);
    void declare_type(std::string name, SP<DataType> type);
    SP<DataType> lookup_type(std::string name);
    void add_command(SP<Command> command);
    std::vector<SP<Command>> &get_commands();

    void add_input(SP<Value> in);
    std::vector<SP<Value>> &get_inputs();
    void add_output(SP<Value> out);
    std::vector<SP<Value>> &get_outputs();

    void auto_add_none();
    void auto_add_inputs();
    void auto_add_outputs();
};

}
}
