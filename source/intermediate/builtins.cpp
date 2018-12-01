#include "builtins.hpp"

#include "data_type.hpp"
#include "scope.hpp"
#include "value.hpp"

namespace waveguide {
namespace intermediate {

std::shared_ptr<Builtins> Builtins::instance{nullptr};

std::shared_ptr<Builtins> Builtins::getInstance() {
    if (!instance) {
        instance = std::shared_ptr<Builtins>(new Builtins());
    }
    return instance;
}
    
Builtins::Builtins()
    : INT{new IntDataType()}, FLOAT{new FloatDataType()}, 
    BOOL{new BoolDataType()}, 
    UPCAST_WILDCARD{new AbstractDataType("UPCAST_WILDCARD")},
    ANY_WILDCARD{new AbstractDataType("ANY_WILDCARD")},
    ADD{new Scope()}, MUL{new Scope()}, RECIP{new Scope()}, MOD{new Scope()},
    BAND{new Scope()}, BOR{new Scope()}, BXOR{new Scope()},
    ITOF{new Scope()}, BTOF{new Scope()}, BTOI{new Scope()}, 
    ITOB{new Scope()}, FTOI{new Scope()}, FTOB{new Scope()},
    COPY{new Scope()}, LOG{new Scope()},
    EQ{new Scope()}, NEQ{new Scope()}, LTE{new Scope()}, GTE{new Scope()},
    LT{new Scope()}, GT{new Scope()}, AND{new Scope()}, OR{new Scope()},
    XOR{new Scope()} {
    
    #define NEW_VALUE(TYPE) std::shared_ptr<Value>(new Value(TYPE))
	ADD->autoAddIns();
	ADD->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	ADD->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	ADD->autoAddOuts();
	ADD->declareVar("x", NEW_VALUE(UPCAST_WILDCARD));

	MUL->autoAddIns();
	MUL->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	MUL->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	MUL->autoAddOuts();
	MUL->declareVar("x", NEW_VALUE(UPCAST_WILDCARD));

	RECIP->autoAddIns();
	RECIP->declareVar("a", NEW_VALUE(FLOAT));
	RECIP->autoAddOuts();
	RECIP->declareVar("x", NEW_VALUE(FLOAT));

	ITOF->autoAddIns();
	ITOF->declareVar("a", NEW_VALUE(INT));
	ITOF->autoAddOuts();
	ITOF->declareVar("x", NEW_VALUE(FLOAT));

	BTOF->autoAddIns();
	BTOF->declareVar("a", NEW_VALUE(BOOL));
	BTOF->autoAddOuts();
	BTOF->declareVar("x", NEW_VALUE(FLOAT));

	BTOI->autoAddIns();
	BTOI->declareVar("a", NEW_VALUE(BOOL));
	BTOI->autoAddOuts();
	BTOI->declareVar("x", NEW_VALUE(INT));

	ITOB->autoAddIns();
	ITOB->declareVar("a", NEW_VALUE(INT));
	ITOB->autoAddOuts();
	ITOB->declareVar("x", NEW_VALUE(BOOL));

	FTOI->autoAddIns();
	FTOI->declareVar("a", NEW_VALUE(FLOAT));
	FTOI->autoAddOuts();
	FTOI->declareVar("x", NEW_VALUE(INT));

	FTOB->autoAddIns();
	FTOB->declareVar("a", NEW_VALUE(FLOAT));
	FTOB->autoAddOuts();
	FTOB->declareVar("x", NEW_VALUE(BOOL));

	// The way this one works is a bit weird. If the input and output are the same size, OFFSET should be zero. The
	// entire value will be copied. If one is bigger than the other, a chunk of data the size of the smaller one will
	// be transferred. OFFSET will be used as the byte index to start copying from or to the larger data type.
	COPY->autoAddIns();
	COPY->declareVar("a", NEW_VALUE(ANY_WILDCARD));
	COPY->declareVar("offset", NEW_VALUE(INT));
	COPY->autoAddOuts();
	COPY->declareVar("x", NEW_VALUE(ANY_WILDCARD));

	LOG->autoAddIns();
	LOG->declareVar("a", NEW_VALUE(ANY_WILDCARD));
	LOG->autoAddOuts();

	MOD->autoAddIns();
	MOD->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	MOD->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	MOD->autoAddOuts();
	MOD->declareVar("x", NEW_VALUE(UPCAST_WILDCARD));

	EQ->autoAddIns();
	EQ->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	EQ->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	EQ->autoAddOuts();
	EQ->declareVar("x", NEW_VALUE(BOOL));

	NEQ->autoAddIns();
	NEQ->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	NEQ->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	NEQ->autoAddOuts();
	NEQ->declareVar("x", NEW_VALUE(BOOL));

	LTE->autoAddIns();
	LTE->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	LTE->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	LTE->autoAddOuts();
	LTE->declareVar("x", NEW_VALUE(BOOL));

	GTE->autoAddIns();
	GTE->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	GTE->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	GTE->autoAddOuts();
	GTE->declareVar("x", NEW_VALUE(BOOL));

	LT->autoAddIns();
	LT->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	LT->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	LT->autoAddOuts();
	LT->declareVar("x", NEW_VALUE(BOOL));

	GT->autoAddIns();
	GT->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	GT->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	GT->autoAddOuts();
	GT->declareVar("x", NEW_VALUE(BOOL));

	AND->autoAddIns();
	AND->declareVar("a", NEW_VALUE(BOOL));
	AND->declareVar("b", NEW_VALUE(BOOL));
	AND->autoAddOuts();
	AND->declareVar("x", NEW_VALUE(BOOL));

	OR->autoAddIns();
	OR->declareVar("a", NEW_VALUE(BOOL));
	OR->declareVar("b", NEW_VALUE(BOOL));
	OR->autoAddOuts();
	OR->declareVar("x", NEW_VALUE(BOOL));

	XOR->autoAddIns();
	XOR->declareVar("a", NEW_VALUE(BOOL));
	XOR->declareVar("b", NEW_VALUE(BOOL));
	XOR->autoAddOuts();
	XOR->declareVar("x", NEW_VALUE(BOOL));

	BAND->autoAddIns();
	BAND->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	BAND->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	BAND->autoAddOuts();
	BAND->declareVar("x", NEW_VALUE(UPCAST_WILDCARD));

	BOR->autoAddIns();
	BOR->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	BOR->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	BOR->autoAddOuts();
	BOR->declareVar("x", NEW_VALUE(UPCAST_WILDCARD));

	BXOR->autoAddIns();
	BXOR->declareVar("a", NEW_VALUE(UPCAST_WILDCARD));
	BXOR->declareVar("b", NEW_VALUE(UPCAST_WILDCARD));
	BXOR->autoAddOuts();
	BXOR->declareVar("x", NEW_VALUE(UPCAST_WILDCARD));
    #undef NEW_VALUE
}

void Builtins::addToScope(std::shared_ptr<Scope> scope) {
    scope->declareType("Int", INT);
    scope->declareType("Float", FLOAT);
    scope->declareType("Bool", BOOL);
    scope->declareType("!UPCAST_WILDCARD", UPCAST_WILDCARD);
    scope->declareType("!ANY_WILDCARD", ANY_WILDCARD);

	scope->declareFunc("!ADD", ADD);
	scope->declareFunc("!MUL", MUL);
	scope->declareFunc("!RECIP", RECIP);
	scope->declareFunc("!ITOF", ITOF);
	scope->declareFunc("!BTOF", BTOF);
	scope->declareFunc("!BTOI", BTOI);
	scope->declareFunc("!ITOB", ITOB);
	scope->declareFunc("!FTOI", FTOI);
	scope->declareFunc("!FTOB", FTOB);
	scope->declareFunc("!COPY", COPY);
	scope->declareFunc("log", LOG);
	scope->declareFunc("!MOD", MOD);
	scope->declareFunc("!EQ", EQ);
	scope->declareFunc("!NEQ", NEQ);
	scope->declareFunc("!LTE", LTE);
	scope->declareFunc("!GTE", GTE);
	scope->declareFunc("!LT", LT);
	scope->declareFunc("!GT", GT);
	scope->declareFunc("!AND", AND);
	scope->declareFunc("!OR", OR);
	scope->declareFunc("!XOR", XOR);
	scope->declareFunc("!BAND", BAND);
	scope->declareFunc("!BOR", BOR);
	scope->declareFunc("!BXOR", BXOR);
}

}
}