#pragma once

#include <memory>

namespace waveguide {
namespace vague {

class builtins;
class template_data_type;
class scope;

using builtins_ptr = std::shared_ptr<builtins>;
using scope_ptr = std::shared_ptr<scope>;
using template_data_type_ptr = std::shared_ptr<template_data_type>;

class builtins {
private:
    builtins();
    static builtins_ptr instance;
public:
    static builtins_ptr get_instance();
    void add_to_scope(scope_ptr scope);
    template_data_type_ptr INT, FLOAT, BOOL, DEDUCE_LATER;
    scope_ptr
        ADD, MUL, RECIP, MOD, BAND, BOR, BXOR,
        ITOF, BTOF, BTOI, ITOB, FTOI, FTOB,
        EQ, NEQ, LTE, GTE, LT, GT, AND, OR, XOR,
        COPY, RETURN,
        LOG, DEF, IF, FOR, FOR_EACH, WHILE;
};

}
}
