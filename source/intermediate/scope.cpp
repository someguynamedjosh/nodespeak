#include "scope.hpp"

#include <sstream>

#include "data_type.hpp"
#include "value.hpp"

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

AugmentationType Augmentation::get_type() {
    return type;
}

std::vector<std::shared_ptr<Value>> &Augmentation::get_params() {
    return params;
}

////////////////////////////////////////////////////////////////////////////////
// Com::Command
////////////////////////////////////////////////////////////////////////////////
Command::Command(std::shared_ptr<Scope> call)
    : call{call} { }

Command::Command(std::shared_ptr<Scope> call, std::shared_ptr<Augmentation> aug)
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
        switch (aug->get_type()) {
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
        for (auto param : aug->get_params()) {
            ss << " " << param->repr();
        }
        ss << " })";
    }
    return ss.str();
}

void Command::add_input(std::shared_ptr<Value> input) {
    ins.push_back(input);
}

void Command::add_output(std::shared_ptr<Value> output) {
    outs.push_back(output);
}

std::vector<std::shared_ptr<Value>> &Command::get_inputs() {
    return ins;
}

std::vector<std::shared_ptr<Value>> &Command::get_outputs() {
    return outs;
}

std::shared_ptr<Augmentation> Command::get_augmentation() {
    return aug;
}

std::shared_ptr<Scope> Command::get_called_scope() {
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
        if (func.second->get_commands().size > 0) {
            ss << func.second->repr() << "\n";
        }
    }

    return ss.str();
}

void Scope::declare_func(std::string name, std::shared_ptr<Scope> body) {
    funcs.emplace(name, body);
}

void Scope::declare_temp_func(std::shared_ptr<Scope> body) {
    tempFuncs.push_back(body);
}

std::shared_ptr<Scope> Scope::lookup_func(std::string name) {
    if (funcs.count(name)) {
        return funcs[name];
    } else if (parent) {
        return parent->lookup_func(name);
    } else {
        return nullptr;
    }
}

void Scope::declare_var(std::string name, std::shared_ptr<Value> value) {
    vars.emplace(name, value);
    if (autoAdd == AutoAdd::INS) {
        add_input(value);
    } else if (autoAdd == AutoAdd::OUTS) {
        add_output(value);
    }
}

void Scope::declare_temp_var(std::shared_ptr<Value> value) {
    tempVars.push_back(value);
}

std::shared_ptr<Value> Scope::lookup_var(std::string name) {
    if (vars.count(name)) {
        return vars[name];
    } else if (parent) {
        return parent->lookup_var(name);
    } else {
        return nullptr;
    }
}

void Scope::declare_type(std::string name, std::shared_ptr<DataType> type) {
    types.emplace(name, type);
}

std::shared_ptr<DataType> Scope::lookup_type(std::string name) {
    if (types.count(name)) {
        return types[name];
    } else if (parent) {
        return parent->lookup_type(name);
    } else {
        return nullptr;
    }
}

void Scope::add_command(std::shared_ptr<Command> command) {
    commands.push_back(command);
}

std::vector<std::shared_ptr<Command>> &Scope::get_commands() {
    return commands;
}


void Scope::add_input(std::shared_ptr<Value> in) {
    ins.push_back(in);
}

std::vector<std::shared_ptr<Value>> &Scope::get_inputs() {
    return ins;
}

void Scope::add_output(std::shared_ptr<Value> out) {
    outs.push_back(out);
}

std::vector<std::shared_ptr<Value>> &Scope::get_outputs() {
    return outs;
}


void Scope::auto_add_none() {
    autoAdd = AutoAdd::NONE;
}

void Scope::auto_add_inputs() {
    autoAdd = AutoAdd::INS;
}

void Scope::auto_add_outputs() {
    autoAdd = AutoAdd::OUTS;
}

}
}