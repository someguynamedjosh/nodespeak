#ifndef _WAVEGUIDE_INTERMEDIATE_SCOPE_H_
#define _WAVEGUIDE_INTERMEDIATE_SCOPE_H_

#include <map>
#include <memory>
#include <string>
#include <vector>

namespace waveguide {
namespace intermediate {

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
    AugmentationType getType();
    std::vector<std::shared_ptr<Value>> &getParams();
};

class Command {
    private:
    std::vector<std::shared_ptr<Value>> ins, outs;
    std::shared_ptr<Augmentation> aug{nullptr};
    Scope &call;
public:
    Command(Scope &call);
    Command(Scope &call, std::shared_ptr<Augmentation> aug);
    std::string repr();

    void addInput(std::shared_ptr<Value> input);
    void addOutput(std::shared_ptr<Value> output);
    std::vector<std::shared_ptr<Value>> &getInputs();
    std::vector<std::shared_ptr<Value>> &getOutputs();
    std::shared_ptr<Augmentation> getAugmentation();
    Scope &getCalledScope();
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

    void declareFunc(std::string name, std::shared_ptr<Scope> body);
    void declareTempFunc(std::shared_ptr<Scope> body);
    std::shared_ptr<Scope> lookupFunc(std::string name);
    void declareVar(std::string name, std::shared_ptr<Value> value);
    void declareTempVar(std::shared_ptr<Value> value);
    std::shared_ptr<Value> lookupVar(std::string name);
    void declareType(std::string name, std::shared_ptr<Value> type);
    std::shared_ptr<DataType> lookupType(std::string name);
    void addCommand(std::shared_ptr<Command> command);
    std::vector<std::shared_ptr<Command>> &getCommands();

    void addIn(std::shared_ptr<Value> in);
    std::vector<std::shared_ptr<Value>> &getIns();
    void addOut(std::shared_ptr<Value> out);
    std::vector<std::shared_ptr<Value>> &getOuts();

    void autoAddNone();
    void autoAddIns();
    void autoAddOuts();
}

}
}

#endif /* _WAVEGUIDE_INTERMEDIATE_SCOPE_H_ */