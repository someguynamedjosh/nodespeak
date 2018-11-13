#include "Scope.h"

#include <sstream>

#include "DataType.h"
#include "Value.h"

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// Com::Augmentation
////////////////////////////////////////////////////////////////////////////////
Augmentation::Augmentation(AugmentationType type)
    : type{type} { }

Augmentation::Augmentation(AugmentationType type, std::shared_ptr<Value> param1)
    : type{type} {
    params.push_back(param1);
}

Augmentation::Augmentation(AugmentationType type, std::shared_ptr<Value> param1,
    std::shared_ptr<Value> param2)
    : type{type} {
    params.push_back(param1);
    params.push_back(param2);
}

AugmentationType Augmentation::getType() {
    return type;
}

std::vector<std::shared_ptr<Value>> &Augmentation::getParams() {
    return params;
}

////////////////////////////////////////////////////////////////////////////////
// Com::Command
////////////////////////////////////////////////////////////////////////////////
Command::Command(Scope &call)
    : call{call} { }

Command::Command(Scope &call, std::shared_ptr<Augmentation> aug)
    : call{call}, aug{aug} { }

std::string Command::repr() {
    std::stringstream ss;
    ss << "COM S@" << (void*) &call << " I={";
    for (auto value : ins) {
        ss << " (" << value->repr() << ")";
    }
    ss << " } O={";
    for (auto value : outs) {
        ss << " (" << value->repr() << ")";
    }
    ss << " }";
    if (aug) {
        ss << " A=(";
        switch (aug->getType()) {
        case AugmentationType::DO_IF:
            ss << "DO_IF";
            break;
        case AugmentationType::DO_IF_NOT:
            ss << "DO_IF_NOT";
            break;
        case AugmentationType::LOOP_FOR:
            ss << "LOOP_FOR";
            break;
        case AugmentationType::LOOP_RANGE:
            ss << "LOOP_RANGE";
            break;
        }
        ss << " {";
        for (auto param : aug->getParams()) {
            ss << " " << param->repr();
        }
        ss << " })";
    }
    return ss.str();
}

void Command::addInput(std::shared_ptr<Value> input) {
    ins.push_back(input);
}

void Command::addOutput(std::shared_ptr<Value> output) {
    outs.push_back(output);
}

std::vector<std::shared_ptr<Value>> &Command::getInputs() {
    return ins;
}

std::vector<std::shared_ptr<Value>> &Command::getOutputs() {
    return outs;
}

std::shared_ptr<Augmentation> Command::getAugmentation() {
    return aug;
}

Scope &Command::getCalledScope() {
    return call;
}

////////////////////////////////////////////////////////////////////////////////
// Com::Scope
////////////////////////////////////////////////////////////////////////////////

Scope::Scope() { }

Scope::Scope(std::shared_ptr<Scope> parent) 
    : parent(parent) { }

std::shared_ptr<Scope> Scope::getParent() {
    return parent;
}

std::string Scope::repr() {
    std::stringstream ss;
    ss << "S@" << (void*) this << " P@" << (void*) parent.get();

    ss << "\nINS={ ";
    for (auto in : ins) {
        ss << "\n" << in->repr();
    }
    ss << "}\nOUTS={ ";
    for (auto out : outs) {
        ss << "\n" << out->repr();
    }

    ss << "}\nTYPES={ ";
    for (auto type : types) {
        ss << "\n\"" << type.first << "\"=" << type.second->repr();
    }

    ss << "}\nFUNCS={ ";
    for (int i = 0; i < tempFuncs.size(); i++) {
        ss << "\n\"!TEMP" << i << "\"=" << tempFuncs[i]->repr();
    }
    for (auto func : funcs) {
        ss << "\n\"" << func.first << "\"=" << func.second->repr();
    }

    ss << "}\nVARS={ ";
    for (int i = 0; i < tempVars.size(); i++) {
        ss << "\n\"!TEMP" << i << "\"=" << tempVars[i]->repr();
    }
    for (auto var : vars) {
        ss << "\n\"" << var.first << "\"=" << var.second->repr();
    }

    ss << "}\nCOMMANDS={ ";
    for (auto command : commands) {
        ss << "\n" + command->repr();
    }

    for (auto func : tempFuncs) {
        ss << func->repr() << "\n";
    }
    for (auto func : funcs) {
        if (func.second->getCommands().size > 0) {
            ss << func.second->repr() << "\n";
        }
    }

    return ss.str();
}

void Scope::declareFunc(std::string name, std::shared_ptr<Scope> body) {
    funcs.emplace(name, body);
}

void Scope::declareTempFunc(std::shared_ptr<Scope> body) {
    tempFuncs.push_back(body);
}

std::shared_ptr<Scope> Scope::lookupFunc(std::string name) {
    if (funcs.count(name)) {
        return funcs[name];
    } else if (parent) {
        return parent->lookupFunc(name);
    } else {
        return nullptr;
    }
}

void Scope::declareVar(std::string name, std::shared_ptr<Value> value) {
    vars.emplace(name, value);
    if (autoAdd == AutoAdd::INS) {
        addIn(value);
    } else if (autoAdd == AutoAdd::OUTS) {
        addOut(value);
    }
}

void Scope::declareTempVar(std::shared_ptr<Value> value) {
    tempVars.push_back(value);
}

std::shared_ptr<Value> Scope::lookupVar(std::string name) {
    if (vars.count(name)) {
        return vars[name];
    } else if (parent) {
        return parent->lookupVar(name);
    } else {
        return nullptr;
    }
}

void Scope::declareType(std::string name, std::shared_ptr<DataType> type) {
    types.emplace(name, type);
}

std::shared_ptr<DataType> Scope::lookupType(std::string name) {
    if (types.count(name)) {
        return types[name];
    } else if (parent) {
        return parent->lookupType(name);
    } else {
        return nullptr;
    }
}

void Scope::addCommand(std::shared_ptr<Command> command) {
    commands.push_back(command);
}

std::vector<std::shared_ptr<Command>> &Scope::getCommands() {
    return commands;
}


void Scope::addIn(std::shared_ptr<Value> in) {
    ins.push_back(in);
}

std::vector<std::shared_ptr<Value>> &Scope::getIns() {
    return ins;
}

void Scope::addOut(std::shared_ptr<Value> out) {
    outs.push_back(out);
}

std::vector<std::shared_ptr<Value>> &Scope::getOuts() {
    return outs;
}


void Scope::autoAddNone() {
    autoAdd = AutoAdd::NONE;
}

void Scope::autoAddIns() {
    autoAdd = AutoAdd::INS;
}

void Scope::autoAddOuts() {
    autoAdd = AutoAdd::OUTS;
}

}
}