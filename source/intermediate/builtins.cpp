#include <waveguide/intermediate/builtins.hpp>

#include <waveguide/intermediate/data_type.hpp>
#include <waveguide/intermediate/scope.hpp>
#include <waveguide/intermediate/value.hpp>

namespace waveguide {
namespace intermediate {

std::shared_ptr<builtins> builtins::instance{nullptr};

std::shared_ptr<builtins> builtins::get_instance() {
    if (!instance) {
        instance = std::shared_ptr<builtins>(new builtins());
    }
    return instance;
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

    COPY{new scope()}, COPY_TO_INDEX{new scope{}}, COPY_FROM_INDEX{new scope{}},
	RETURN{new scope()},

	LOG{new scope()}, DEF{new scope()}, IF{new scope()}, FOR{new scope()}, 
	FOR_EACH{new scope()}, WHILE{new scope()} {
    
    #define NEW_VALUE(TYPE) std::shared_ptr<value>(new value(TYPE))
	ADD->auto_add_inputs();
	ADD->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	ADD->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	ADD->auto_add_outputs();
	ADD->declare_var("x", NEW_VALUE(DEDUCE_LATER));

	MUL->auto_add_inputs();
	MUL->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	MUL->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	MUL->auto_add_outputs();
	MUL->declare_var("x", NEW_VALUE(DEDUCE_LATER));

	RECIP->auto_add_inputs();
	RECIP->declare_var("a", NEW_VALUE(FLOAT));
	RECIP->auto_add_outputs();
	RECIP->declare_var("x", NEW_VALUE(FLOAT));

	MOD->auto_add_inputs();
	MOD->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	MOD->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	MOD->auto_add_outputs();
	MOD->declare_var("x", NEW_VALUE(DEDUCE_LATER));

	BAND->auto_add_inputs();
	BAND->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	BAND->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	BAND->auto_add_outputs();
	BAND->declare_var("x", NEW_VALUE(DEDUCE_LATER));

	BOR->auto_add_inputs();
	BOR->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	BOR->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	BOR->auto_add_outputs();
	BOR->declare_var("x", NEW_VALUE(DEDUCE_LATER));

	BXOR->auto_add_inputs();
	BXOR->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	BXOR->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	BXOR->auto_add_outputs();
	BXOR->declare_var("x", NEW_VALUE(DEDUCE_LATER));

	ITOF->auto_add_inputs();
	ITOF->declare_var("a", NEW_VALUE(INT));
	ITOF->auto_add_outputs();
	ITOF->declare_var("x", NEW_VALUE(FLOAT));

	BTOF->auto_add_inputs();
	BTOF->declare_var("a", NEW_VALUE(BOOL));
	BTOF->auto_add_outputs();
	BTOF->declare_var("x", NEW_VALUE(FLOAT));

	BTOI->auto_add_inputs();
	BTOI->declare_var("a", NEW_VALUE(BOOL));
	BTOI->auto_add_outputs();
	BTOI->declare_var("x", NEW_VALUE(INT));

	ITOB->auto_add_inputs();
	ITOB->declare_var("a", NEW_VALUE(INT));
	ITOB->auto_add_outputs();
	ITOB->declare_var("x", NEW_VALUE(BOOL));

	FTOI->auto_add_inputs();
	FTOI->declare_var("a", NEW_VALUE(FLOAT));
	FTOI->auto_add_outputs();
	FTOI->declare_var("x", NEW_VALUE(INT));

	FTOB->auto_add_inputs();
	FTOB->declare_var("a", NEW_VALUE(FLOAT));
	FTOB->auto_add_outputs();
	FTOB->declare_var("x", NEW_VALUE(BOOL));

	EQ->auto_add_inputs();
	EQ->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	EQ->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	EQ->auto_add_outputs();
	EQ->declare_var("x", NEW_VALUE(BOOL));

	NEQ->auto_add_inputs();
	NEQ->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	NEQ->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	NEQ->auto_add_outputs();
	NEQ->declare_var("x", NEW_VALUE(BOOL));

	LTE->auto_add_inputs();
	LTE->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	LTE->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	LTE->auto_add_outputs();
	LTE->declare_var("x", NEW_VALUE(BOOL));

	GTE->auto_add_inputs();
	GTE->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	GTE->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	GTE->auto_add_outputs();
	GTE->declare_var("x", NEW_VALUE(BOOL));

	LT->auto_add_inputs();
	LT->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	LT->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	LT->auto_add_outputs();
	LT->declare_var("x", NEW_VALUE(BOOL));

	GT->auto_add_inputs();
	GT->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	GT->declare_var("b", NEW_VALUE(DEDUCE_LATER));
	GT->auto_add_outputs();
	GT->declare_var("x", NEW_VALUE(BOOL));

	AND->auto_add_inputs();
	AND->declare_var("a", NEW_VALUE(BOOL));
	AND->declare_var("b", NEW_VALUE(BOOL));
	AND->auto_add_outputs();
	AND->declare_var("x", NEW_VALUE(BOOL));

	OR->auto_add_inputs();
	OR->declare_var("a", NEW_VALUE(BOOL));
	OR->declare_var("b", NEW_VALUE(BOOL));
	OR->auto_add_outputs();
	OR->declare_var("x", NEW_VALUE(BOOL));

	XOR->auto_add_inputs();
	XOR->declare_var("a", NEW_VALUE(BOOL));
	XOR->declare_var("b", NEW_VALUE(BOOL));
	XOR->auto_add_outputs();
	XOR->declare_var("x", NEW_VALUE(BOOL));

	COPY->auto_add_inputs();
	COPY->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	COPY->auto_add_outputs();
	COPY->declare_var("x", NEW_VALUE(DEDUCE_LATER));

	COPY_TO_INDEX->auto_add_inputs();
	COPY_TO_INDEX->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	COPY_TO_INDEX->declare_var("index", NEW_VALUE(INT));
	COPY_TO_INDEX->auto_add_outputs();
	COPY_TO_INDEX->declare_var("x", NEW_VALUE(DEDUCE_LATER));

	COPY_FROM_INDEX->auto_add_inputs();
	COPY_FROM_INDEX->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	COPY_FROM_INDEX->declare_var("index", NEW_VALUE(INT));
	COPY_FROM_INDEX->auto_add_outputs();
	COPY_FROM_INDEX->declare_var("x", NEW_VALUE(DEDUCE_LATER));

	// RETURN has no inputs, no outputs.

	LOG->auto_add_inputs();
	LOG->declare_var("a", NEW_VALUE(DEDUCE_LATER));
	LOG->auto_add_outputs();

	// DEF has no inputs, no outputs.

	IF->auto_add_inputs();
	IF->declare_var("condition", NEW_VALUE(BOOL));
	IF->auto_add_outputs();
	IF->declare_var("return", NEW_VALUE(DEDUCE_LATER));

	FOR->auto_add_inputs();
	FOR->declare_var("times", NEW_VALUE(INT));
	FOR->auto_add_outputs();
	FOR->declare_var("return", NEW_VALUE(DEDUCE_LATER));

	FOR_EACH->auto_add_inputs();
	FOR_EACH->declare_var("times", NEW_VALUE(INT));
	FOR_EACH->auto_add_outputs();
	FOR_EACH->declare_var("return", NEW_VALUE(DEDUCE_LATER));

	WHILE->auto_add_inputs();
	WHILE->declare_var("condition", NEW_VALUE(BOOL));
	WHILE->auto_add_outputs();
	WHILE->declare_var("return", NEW_VALUE(DEDUCE_LATER));

    #undef NEW_VALUE
}

void builtins::add_to_scope(std::shared_ptr<scope> scope) {
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
	scope->declare_func("!COPY_TO_INDEX", COPY_TO_INDEX);
	scope->declare_func("!COPY_FROM_INDEX", COPY_FROM_INDEX);
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