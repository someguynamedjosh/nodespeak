#pragma once

#include <map>
#include <memory>
#include <string>
#include <vector>

namespace waveguide {
namespace intermediate {

class DataType;
class Scope;
class Value;

enum AugmentationType {
	DO_IF, DO_IF_NOT, LOOP_FOR, LOOP_RANGE
};

class Augmentation {
private:
    AugmentationType type;
    std::vector<std::shared_ptr<Value>> params;
public:
    Augmentation(AugmentationType type);
    Augmentation(AugmentationType type, std::shared_ptr<Value> param1);
    Augmentation(AugmentationType type, std::shared_ptr<Value> param1,
        std::shared_ptr<Value> param2);
    AugmentationType get_type();
    std::vector<std::shared_ptr<Value>> &get_params();
};

class Command {
    private:
    std::vector<std::shared_ptr<Value>> ins, outs;
    std::shared_ptr<Scope> call{nullptr};
    std::shared_ptr<Augmentation> aug{nullptr};
public:
    Command(std::shared_ptr<Scope> call);
    Command(std::shared_ptr<Scope> call, std::shared_ptr<Augmentation> aug);
    std::string repr();

    void add_input(std::shared_ptr<Value> input);
    void add_output(std::shared_ptr<Value> output);
    std::vector<std::shared_ptr<Value>> &get_inputs();
    std::vector<std::shared_ptr<Value>> &get_outputs();
    std::shared_ptr<Augmentation> get_augmentation();
    std::shared_ptr<Scope> get_called_scope();
};

class Scope {
private:
    std::map<std::string, std::shared_ptr<Scope>> funcs;
    std::vector<std::shared_ptr<Scope>> tempFuncs;
    std::map<std::string, std::shared_ptr<Value>> vars;
    std::vector<std::shared_ptr<Value>> tempVars;
    std::map<std::string, std::shared_ptr<DataType>> types;
    std::vector<std::shared_ptr<Command>> commands;
    std::shared_ptr<Scope> parent{nullptr};

    enum AutoAdd {
        NONE, INS, OUTS
    };
    AutoAdd autoAdd{AutoAdd::NONE};
    std::vector<std::shared_ptr<Value>> ins, outs;
public:
    Scope();
    Scope(std::shared_ptr<Scope> parent);
    std::shared_ptr<Scope> getParent();
    std::string repr();

    void declare_func(std::string name, std::shared_ptr<Scope> body);
    void declare_temp_func(std::shared_ptr<Scope> body);
    std::shared_ptr<Scope> lookup_func(std::string name);
    void declare_var(std::string name, std::shared_ptr<Value> value);
    void declare_temp_var(std::shared_ptr<Value> value);
    std::shared_ptr<Value> lookup_var(std::string name);
    void declare_type(std::string name, std::shared_ptr<DataType> type);
    std::shared_ptr<DataType> lookup_type(std::string name);
    void add_command(std::shared_ptr<Command> command);
    std::vector<std::shared_ptr<Command>> &get_commands();

    void add_input(std::shared_ptr<Value> in);
    std::vector<std::shared_ptr<Value>> &get_inputs();
    void add_output(std::shared_ptr<Value> out);
    std::vector<std::shared_ptr<Value>> &get_outputs();

    void auto_add_none();
    void auto_add_inputs();
    void auto_add_outputs();
};

}
}
