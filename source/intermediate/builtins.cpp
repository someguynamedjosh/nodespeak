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
    UPCAST_WILDCARD{new abstract_data_type("UPCAST_WILDCARD")},
    ANY_WILDCARD{new abstract_data_type("ANY_WILDCARD")},
    ADD{new scope()}, MUL{new scope()}, RECIP{new scope()}, MOD{new scope()},
    BAND{new scope()}, BOR{new scope()}, BXOR{new scope()},
    ITOF{new scope()}, BTOF{new scope()}, BTOI{new scope()}, 
    ITOB{new scope()}, FTOI{new scope()}, FTOB{new scope()},
    COPY{new scope()}, LOG{new scope()}, RETURN{new scope()},
    EQ{new scope()}, NEQ{new scope()}, LTE{new scope()}, GTE{new scope()},
    LT{new scope()}, GT{new scope()}, AND{new scope()}, OR{new scope()},
    XOR{new scope()} {
    
    #define NEW_VALUE(TYPE) std::shared_ptr<value>(new value(TYPE))
	ADD->auto_add_inputs();
	ADD->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	ADD->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
	ADD->auto_add_outputs();
	ADD->declare_var("x", NEW_VALUE(UPCAST_WILDCARD));

	MUL->auto_add_inputs();
	MUL->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	MUL->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
	MUL->auto_add_outputs();
	MUL->declare_var("x", NEW_VALUE(UPCAST_WILDCARD));

	RECIP->auto_add_inputs();
	RECIP->declare_var("a", NEW_VALUE(FLOAT));
	RECIP->auto_add_outputs();
	RECIP->declare_var("x", NEW_VALUE(FLOAT));

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

	// The way this one works is a bit weird. If the input and output are the same size, OFFSET should be zero. The
	// entire value will be copied. If one is bigger than the other, a chunk of data the size of the smaller one will
	// be transferred. OFFSET will be used as the byte index to start copying from or to the larger data type.
	COPY->auto_add_inputs();
	COPY->declare_var("a", NEW_VALUE(ANY_WILDCARD));
	COPY->declare_var("offset", NEW_VALUE(INT));
	COPY->auto_add_outputs();
	COPY->declare_var("x", NEW_VALUE(ANY_WILDCARD));

	LOG->auto_add_inputs();
	LOG->declare_var("a", NEW_VALUE(ANY_WILDCARD));
	LOG->auto_add_outputs();

	// RETURN has no inputs, no outputs.

	MOD->auto_add_inputs();
	MOD->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	MOD->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
	MOD->auto_add_outputs();
	MOD->declare_var("x", NEW_VALUE(UPCAST_WILDCARD));

	EQ->auto_add_inputs();
	EQ->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	EQ->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
	EQ->auto_add_outputs();
	EQ->declare_var("x", NEW_VALUE(BOOL));

	NEQ->auto_add_inputs();
	NEQ->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	NEQ->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
	NEQ->auto_add_outputs();
	NEQ->declare_var("x", NEW_VALUE(BOOL));

	LTE->auto_add_inputs();
	LTE->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	LTE->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
	LTE->auto_add_outputs();
	LTE->declare_var("x", NEW_VALUE(BOOL));

	GTE->auto_add_inputs();
	GTE->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	GTE->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
	GTE->auto_add_outputs();
	GTE->declare_var("x", NEW_VALUE(BOOL));

	LT->auto_add_inputs();
	LT->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	LT->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
	LT->auto_add_outputs();
	LT->declare_var("x", NEW_VALUE(BOOL));

	GT->auto_add_inputs();
	GT->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	GT->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
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

	BAND->auto_add_inputs();
	BAND->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	BAND->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
	BAND->auto_add_outputs();
	BAND->declare_var("x", NEW_VALUE(UPCAST_WILDCARD));

	BOR->auto_add_inputs();
	BOR->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	BOR->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
	BOR->auto_add_outputs();
	BOR->declare_var("x", NEW_VALUE(UPCAST_WILDCARD));

	BXOR->auto_add_inputs();
	BXOR->declare_var("a", NEW_VALUE(UPCAST_WILDCARD));
	BXOR->declare_var("b", NEW_VALUE(UPCAST_WILDCARD));
	BXOR->auto_add_outputs();
	BXOR->declare_var("x", NEW_VALUE(UPCAST_WILDCARD));
    #undef NEW_VALUE
}

void builtins::add_to_scope(std::shared_ptr<scope> scope) {
    scope->declare_type("Int", INT);
    scope->declare_type("Float", FLOAT);
    scope->declare_type("Bool", BOOL);
    scope->declare_type("!UPCAST_WILDCARD", UPCAST_WILDCARD);
    scope->declare_type("!ANY_WILDCARD", ANY_WILDCARD);

	scope->declare_func("!ADD", ADD);
	scope->declare_func("!MUL", MUL);
	scope->declare_func("!RECIP", RECIP);
	scope->declare_func("!ITOF", ITOF);
	scope->declare_func("!BTOF", BTOF);
	scope->declare_func("!BTOI", BTOI);
	scope->declare_func("!ITOB", ITOB);
	scope->declare_func("!FTOI", FTOI);
	scope->declare_func("!FTOB", FTOB);
	scope->declare_func("!COPY", COPY);
	scope->declare_func("log", LOG);
	scope->declare_func("!MOD", MOD);
	scope->declare_func("!EQ", EQ);
	scope->declare_func("!NEQ", NEQ);
	scope->declare_func("!LTE", LTE);
	scope->declare_func("!GTE", GTE);
	scope->declare_func("!LT", LT);
	scope->declare_func("!GT", GT);
	scope->declare_func("!AND", AND);
	scope->declare_func("!OR", OR);
	scope->declare_func("!XOR", XOR);
	scope->declare_func("!BAND", BAND);
	scope->declare_func("!BOR", BOR);
	scope->declare_func("!BXOR", BXOR);
}

}
}