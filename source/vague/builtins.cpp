#include <waveguide/vague/builtins.hpp>

#include <waveguide/vague/scope.hpp>
#include <waveguide/vague/type_template.hpp>
#include <waveguide/vague/value.hpp>

#include "util/aliases.hpp"

namespace waveguide {
namespace vague {

builtins_ptr builtins::instance{nullptr};

builtins_ptr builtins::get_instance() {
    if (!instance) {
        instance = std::shared_ptr<builtins>{new builtins{}};
    }
    return instance;
}

void add_ax_io(
	scope_ptr add_to, template_data_type_ptr in_type_template, 
	template_data_type_ptr out_type_template
) {
	add_to->add_input("a", in_type_template);
	add_to->add_output("x", out_type_template);

}

void add_abx_io(
	scope_ptr add_to, template_data_type_ptr in_type_template, 
	template_data_type_ptr out_type_template
) {
	add_to->add_input("a", in_type_template);
	add_to->add_input("b", in_type_template);
	add_to->add_output("x", out_type_template);
}
    
builtins::builtins()
    : INT{new template_named_data_type("Int")}, 
	FLOAT{new template_named_data_type("Float")}, 
	BOOL{new template_named_data_type("Bool")}, 
	// TODO: Implement abstract template data type.
    DEDUCE_LATER{new template_wildcard_data_type("!DEDUCE_LATER")},

    ADD{new scope()}, MUL{new scope()}, RECIP{new scope()}, MOD{new scope()},
    BAND{new scope()}, BOR{new scope()}, BXOR{new scope()},

    ITOF{new scope()}, BTOF{new scope()}, BTOI{new scope()}, 
    ITOB{new scope()}, FTOI{new scope()}, FTOB{new scope()},

    EQ{new scope()}, NEQ{new scope()}, LTE{new scope()}, GTE{new scope()},
    LT{new scope()}, GT{new scope()}, AND{new scope()}, OR{new scope()},
    XOR{new scope()},

    COPY{new scope()}, RETURN{new scope()},

	LOG{new scope()}, DEF{new scope()}, IF{new scope()}, FOR{new scope()}, 
	FOR_EACH{new scope()}, WHILE{new scope()} {
    
	// ABX IO is two inputs (a, b) with one data type and one output (x) with
	// another data type.
	// Uniform ABX IO is two inputs (a, b) and one output (x) that should all 
	// have the same (unknown / templated) data type.
	// AX IO is one input (a) and one output (x).
	auto wildcard{std::make_shared<template_wildcard_data_type>("!TYPE")};
	auto wildcard2{std::make_shared<template_wildcard_data_type>("!TYPE2")};

	add_abx_io(ADD, wildcard, wildcard);
	add_abx_io(MUL, wildcard, wildcard);
	add_abx_io(RECIP, FLOAT, FLOAT);
	add_abx_io(MOD, wildcard, wildcard);
	add_abx_io(BAND, wildcard, wildcard);
	add_abx_io(BOR, wildcard, wildcard);
	add_abx_io(BXOR, wildcard, wildcard);

	add_ax_io(ITOF, INT, FLOAT);
	add_ax_io(BTOF, BOOL, FLOAT);
	add_ax_io(BTOI, BOOL, INT);
	add_ax_io(ITOB, INT, BOOL);
	add_ax_io(FTOI, FLOAT, INT);
	add_ax_io(FTOB, FLOAT, BOOL);

	add_abx_io(EQ, wildcard, BOOL);
	add_abx_io(NEQ, wildcard, BOOL);
	add_abx_io(LTE, wildcard, BOOL);
	add_abx_io(GTE, wildcard, BOOL);
	add_abx_io(LT, wildcard, BOOL);
	add_abx_io(GT, wildcard, BOOL);

	add_abx_io(AND, BOOL, BOOL);
	add_abx_io(OR, BOOL, BOOL);
	add_abx_io(XOR, BOOL, BOOL);

	add_ax_io(COPY, wildcard, wildcard);
	// RETURN has no inputs, no outputs.
	LOG->add_input("a", wildcard);
	// DEF has no inputs, no outputs.
	IF->add_input("condition", BOOL);
	FOR->add_input("times", INT);
	FOR_EACH->add_input("values", wildcard);
	WHILE->add_input("condition", BOOL);
}

void builtins::add_to_scope(scope_ptr scope) {
    scope->declare_type("Int", INT);
    scope->declare_type("Float", FLOAT);
    scope->declare_type("Bool", BOOL);
    scope->declare_type("!DEDUCE_LATER", DEDUCE_LATER);

	scope->declare_func("!ADD", ADD);
	scope->declare_func("!MUL", MUL);
	scope->declare_func("!RECIP", RECIP);
	scope->declare_func("!MOD", MOD);
	scope->declare_func("!BAND", BAND);
	scope->declare_func("!BOR", BOR);
	scope->declare_func("!BXOR", BXOR);

	scope->declare_func("!ITOF", ITOF);
	scope->declare_func("!BTOF", BTOF);
	scope->declare_func("!BTOI", BTOI);
	scope->declare_func("!ITOB", ITOB);
	scope->declare_func("!FTOI", FTOI);
	scope->declare_func("!FTOB", FTOB);

	scope->declare_func("!EQ", EQ);
	scope->declare_func("!NEQ", NEQ);
	scope->declare_func("!LTE", LTE);
	scope->declare_func("!GTE", GTE);
	scope->declare_func("!LT", LT);
	scope->declare_func("!GT", GT);
	scope->declare_func("!AND", AND);
	scope->declare_func("!OR", OR);
	scope->declare_func("!XOR", XOR);

	scope->declare_func("!COPY", COPY);
	scope->declare_func("!RETURN", RETURN);

	scope->declare_func("log", LOG);
	scope->declare_func("def", DEF);
	scope->declare_func("if", IF);
	scope->declare_func("for", FOR);
	scope->declare_func("for_each", FOR_EACH);
	scope->declare_func("while", WHILE);
}

}
}