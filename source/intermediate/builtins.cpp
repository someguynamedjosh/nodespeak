#include <waveguide/intermediate/builtins.hpp>

#include <waveguide/intermediate/data_type.hpp>
#include <waveguide/intermediate/scope.hpp>
#include <waveguide/intermediate/type_template.hpp>
#include <waveguide/intermediate/value.hpp>

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

builtins_ptr builtins::instance{nullptr};

builtins_ptr builtins::get_instance() {
    if (!instance) {
        instance = std::shared_ptr<builtins>{new builtins{}};
    }
    return instance;
}

void add_ax_io(scope_ptr add_to, std::string in_type, std::string out_type) {
	vague_data_type_ptr in_type_template = std::make_shared<vague_basic_data_type>(in_type);
	vague_data_type_ptr out_type_template = std::make_shared<vague_basic_data_type>(out_type);
	add_to->add_input("a", in_type_template);
	add_to->add_output("x", out_type_template);

}

void add_abx_io(scope_ptr add_to, std::string in_type, std::string out_type) {
	vague_data_type_ptr in_type_template{new vague_basic_data_type{in_type}};
	vague_data_type_ptr out_type_template{new vague_basic_data_type{out_type}};
	add_to->add_input("a", in_type_template);
	add_to->add_input("b", in_type_template);
	add_to->add_output("x", out_type_template);
}

void add_uniform_abx_io(scope_ptr add_to) {
	add_abx_io(add_to, "!TYPE", "!TYPE");
}
    
builtins::builtins()
    : INT{new int_data_type()}, FLOAT{new float_data_type()}, 
    BOOL{new bool_data_type()}, 
    DEDUCE_LATER{new abstract_data_type("DEDUCE_LATER")},

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
	add_uniform_abx_io(ADD);
	add_uniform_abx_io(MUL);
	add_ax_io(RECIP, "Float", "Float");
	add_uniform_abx_io(MOD);
	add_uniform_abx_io(BAND);
	add_uniform_abx_io(BOR);
	add_uniform_abx_io(BXOR);

	add_ax_io(ITOF, "Int", "Float");
	add_ax_io(BTOF, "Bool", "Float");
	add_ax_io(BTOI, "Bool", "Int");
	add_ax_io(ITOB, "Int", "Bool");
	add_ax_io(FTOI, "Float", "Int");
	add_ax_io(FTOB, "Float", "Bool");

	add_abx_io(EQ, "!TYPE", "Bool");
	add_abx_io(NEQ, "!TYPE", "Bool");
	add_abx_io(LTE, "!TYPE", "Bool");
	add_abx_io(GTE, "!TYPE", "Bool");
	add_abx_io(LT, "!TYPE", "Bool");
	add_abx_io(GT, "!TYPE", "Bool");

	add_abx_io(AND, "Bool", "Bool");
	add_abx_io(OR, "Bool", "Bool");
	add_abx_io(XOR, "Bool", "Bool");

	vague_data_type_ptr wildcard_type{new vague_basic_data_type{"!TYPE"}};
	vague_data_type_ptr wildcard2_type{new vague_basic_data_type{"!TYPE2"}};
	vague_data_type_ptr int_type{new vague_basic_data_type{"Int"}};
	vague_data_type_ptr bool_type{new vague_basic_data_type{"Bool"}};
	add_ax_io(COPY, "!TYPE", "!TYPE");
	// RETURN has no inputs, no outputs.

	LOG->add_input("a", wildcard_type);
	// DEF has no inputs, no outputs.
	IF->add_input("condition", bool_type);
	FOR->add_input("times", int_type);
	FOR_EACH->add_input("values", wildcard2_type);
	WHILE->add_input("condition", bool_type);
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