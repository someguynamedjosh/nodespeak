#include <waveguide/intermediate/data_type.hpp>
#include <waveguide/intermediate/scope.hpp>
#include <waveguide/intermediate/value.hpp>
#include <sstream>

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// Com::command
////////////////////////////////////////////////////////////////////////////////
command::command(std::shared_ptr<scope> call)
    : call{call} { }

command::command(std::shared_ptr<scope> call, std::shared_ptr<augmentation> aug)
    : call{call}, aug{aug} { }

std::string command::repr() {
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
        ss << " })";
    }
    return ss.str();
}

void command::add_input(std::shared_ptr<value> input) {
    ins.push_back(input);
}

void command::add_output(std::shared_ptr<value> output) {
    outs.push_back(output);
}

const std::vector<std::shared_ptr<value>> &command::get_inputs() const {
    return ins;
}

const std::vector<std::shared_ptr<value>> &command::get_outputs() const {
    return outs;
}

const std::shared_ptr<augmentation> command::get_augmentation() const {
    return aug;
}

const std::shared_ptr<scope> command::get_called_scope() const {
    return call;
}

////////////////////////////////////////////////////////////////////////////////
// Com::scope
////////////////////////////////////////////////////////////////////////////////

scope::scope() { }

scope::scope(std::shared_ptr<scope> parent) 
    : parent(parent) { }

const std::shared_ptr<scope> scope::get_parent() const {
    return parent;
}

std::string scope::repr() {
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
    for (uint i = 0; i < tempFuncs.size(); i++) {
        ss << "\n\"!TEMP" << i << "\"=" << tempFuncs[i]->repr();
    }
    for (auto func : funcs) {
        ss << "\n\"" << func.first << "\"=" << func.second->repr();
    }

    ss << "}\nVARS={ ";
    for (uint i = 0; i < tempVars.size(); i++) {
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
        if (func.second->get_commands().size() > 0) {
            ss << func.second->repr() << "\n";
        }
    }

    return ss.str();
}

void scope::declare_func(std::string name, std::shared_ptr<scope> body) {
    funcs.emplace(name, body);
}

void scope::declare_temp_func(std::shared_ptr<scope> body) {
    tempFuncs.push_back(body);
}

const std::shared_ptr<scope> scope::lookup_func(std::string name) const {
    if (funcs.count(name)) {
        return funcs.at(name);
    } else if (parent) {
        return parent->lookup_func(name);
    } else {
        return nullptr;
    }
}

void scope::declare_var(std::string name, std::shared_ptr<value> value) {
    vars.emplace(name, value);
    if (do_auto == auto_add::INS) {
        add_input(value);
    } else if (do_auto == auto_add::OUTS) {
        add_output(value);
    }
}

void scope::declare_temp_var(std::shared_ptr<value> value) {
    tempVars.push_back(value);
}

const std::shared_ptr<value> scope::lookup_var(std::string name) const {
    if (vars.count(name)) {
        return vars.at(name);
    } else if (parent) {
        return parent->lookup_var(name);
    } else {
        return nullptr;
    }
}

void scope::declare_type(std::string name, std::shared_ptr<data_type> type) {
    types.emplace(name, type);
}

const std::shared_ptr<data_type> scope::lookup_type(std::string name) const {
    if (types.count(name)) {
        return types.at(name);
    } else if (parent) {
        return parent->lookup_type(name);
    } else {
        return nullptr;
    }
}

void scope::add_command(std::shared_ptr<command> command) {
    commands.push_back(command);
}

const std::vector<std::shared_ptr<command>> &scope::get_commands() const {
    return commands;
}


void scope::add_input(std::shared_ptr<value> in) {
    ins.push_back(in);
}

const std::vector<std::shared_ptr<value>> &scope::get_inputs() const {
    return ins;
}

void scope::add_output(std::shared_ptr<value> out) {
    outs.push_back(out);
}

const std::vector<std::shared_ptr<value>> &scope::get_outputs() const {
    return outs;
}


void scope::auto_add_none() {
    do_auto = auto_add::NONE;
}

void scope::auto_add_inputs() {
    do_auto = auto_add::INS;
}

void scope::auto_add_outputs() {
    do_auto = auto_add::OUTS;
}

}
}