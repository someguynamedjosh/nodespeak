#include <iostream>
#include <sstream>
#include "scope.h"
#include "tokens.h"

namespace Com {

FuncScope *Scope::lookupFunc(string name) {
	if(funcs.count(name)) {
		return funcs[name];
	} else if(parent != nullptr) {
		return parent->lookupFunc(name);
	} else {
		return nullptr;
	}
}

Variable *Scope::lookupVar(string name) {
	if(vars.count(name)) {
		return vars[name];
	} else if(parent != nullptr) {
		return parent->lookupVar(name);
	} else {
		return nullptr;
	}
}

DataType *Scope::lookupType(string name) {
	if(types.count(name)) {
		return types[name];
	} else if(parent != nullptr) {
		return parent->lookupType(name);
	} else {
		return nullptr;
	}
}

void FuncScope::declareVar(string name, Variable *variable) {
	Scope::declareVar(name, variable);
	if (autoAdd == 1) {
		ins.push_back(variable);
	} else if (autoAdd == 2) {
		outs.push_back(variable);
	}
}

string Literal::repr(DataType *dtype, void *data) { 
	stringstream ss;
	ss << "lit " << (void*) dtype << " ";
	if (dtype == DATA_TYPE_FLOAT) {
		ss << *((float*) data);
	} else if (dtype == DATA_TYPE_INT) {
		ss << *((int*) data);
	} else if (dtype == DATA_TYPE_BOOL) {
		ss << *((bool*) data);
	} else if (dtype->getArrayDepth() > 0) {
		ArrayDataType *atype = dynamic_cast<ArrayDataType*>(dtype);
		DataType *stype = atype->getBaseType();
		ss << "[";
		for (int i = 0; i < atype->getArrayLength(); i++) {
			ss << repr(stype, ((char*) data) + i * stype->getLength()) << ", ";
		}
		ss << "]";
	} else {
		ss << data;
	}
	return ss.str();
}

string Literal::repr() {
	return repr(type, data);
}

string Variable::repr() {
	stringstream ss;
	ss << "var " << (void*) type;
	ss << " " << (void*) this;
	return ss.str();
}

string FuncInput::repr() {
	if (varIn) {
		return varIn->repr();
	} else {
		return litIn->repr();
	}
}

string FuncCommand::repr() {
	stringstream ss;
	ss << "funcc " << (void*) call;
	for (FuncInput *input : ins) {
		ss << " " << input->repr();
	}
	ss << ",";
	for (Variable *output : outs) {
		ss << " " << output->repr();
	}
	return ss.str();
}

DataType *DATA_TYPE_INT, *DATA_TYPE_FLOAT, *DATA_TYPE_BOOL;
FuncScope *BUILTIN_ADD, *BUILTIN_MUL, *BUILTIN_RECIP, *BUILTIN_MOD;
FuncScope *BUILTIN_ITOF, *BUILTIN_BTOF, *BUILTIN_BTOI, *BUILTIN_ITOB, *BUILTIN_FTOI, *BUILTIN_FTOB;
FuncScope *BUILTIN_EQ, *BUILTIN_NEQ, *BUILTIN_LTE, *BUILTIN_GTE, *BUILTIN_LT, *BUILTIN_GT;
FuncScope *BUILTIN_AND, *BUILTIN_OR, *BUILTIN_XOR, *BUILTIN_BAND, *BUILTIN_BOR, *BUILTIN_BXOR;
FuncScope *BUILTIN_COPY, *BUILTIN_INDEX;
Literal *evalBuiltinFunc(FuncScope *func, Literal *a, Literal *b) {
	if (b == nullptr) { // There is no second argument, don't do any type conversions.
	} else if (a->getType() == DATA_TYPE_FLOAT) {
		if (b->getType() == DATA_TYPE_INT) {
			b = new Literal(DATA_TYPE_FLOAT, new float(*b->interpretAsFloat()));
		} else if (b->getType() == DATA_TYPE_BOOL) {
			b = new Literal(DATA_TYPE_FLOAT, new float((*b->interpretAsBool()) ? 1.0f : 0.0f));
		}
	} else if (a->getType() == DATA_TYPE_INT) {
		if (b->getType() == DATA_TYPE_FLOAT) {
			a = new Literal(DATA_TYPE_FLOAT, new float(*a->interpretAsInt()));
		} else if (b->getType() == DATA_TYPE_BOOL) {
			b = new Literal(DATA_TYPE_INT, new int((*b->interpretAsBool()) ? 1 : 0));
		}
	} else if (a->getType() == DATA_TYPE_BOOL) {
		if (b->getType() == DATA_TYPE_FLOAT) {
			a = new Literal(DATA_TYPE_FLOAT, new float((*a->interpretAsBool()) ? 1.0f : 0.0f));
		} else if (b->getType() == DATA_TYPE_INT) {
			a = new Literal(DATA_TYPE_INT, new int((*a->interpretAsBool()) ? 1 : 0));
		}
	}

	DataType *d = a->getType();
	FuncScope *f = func;
	if (d == DATA_TYPE_FLOAT) {
		if (f == BUILTIN_ADD) {
			return new Literal(DATA_TYPE_FLOAT, new float(*a->interpretAsFloat() + *b->interpretAsFloat()));
		} else if (f == BUILTIN_MUL) {
			return new Literal(DATA_TYPE_FLOAT, new float(*a->interpretAsFloat() * *b->interpretAsFloat()));
		} else if (f == BUILTIN_RECIP) {
			return new Literal(DATA_TYPE_FLOAT, new float(1.0f / *a->interpretAsFloat()));
		}
	} else if (d == DATA_TYPE_INT) {
		if (f == BUILTIN_ADD) {
			return new Literal(DATA_TYPE_INT, new int(*a->interpretAsInt() + *b->interpretAsInt()));
		} else if (f == BUILTIN_MUL) {
			return new Literal(DATA_TYPE_INT, new int(*a->interpretAsInt() * *b->interpretAsInt()));
		} else if (f == BUILTIN_RECIP) {
			return new Literal(DATA_TYPE_INT, new int(1.0f / *a->interpretAsInt()));
		}
	} else if (d == DATA_TYPE_BOOL) {
		if (f == BUILTIN_ADD) {
			return new Literal(DATA_TYPE_BOOL, new bool(*a->interpretAsBool() ^ *b->interpretAsBool()));
		} else if (f == BUILTIN_MUL) {
			return new Literal(DATA_TYPE_BOOL, new bool(*a->interpretAsBool() && *b->interpretAsBool()));
		} else if (f == BUILTIN_RECIP) {
			return new Literal(DATA_TYPE_BOOL, new bool(*a->interpretAsBool()));
		}
	}
	return nullptr;
}

string Scope::repr() {
	stringstream ss;
	ss << "===SCOPE at " << (void*) this << "===" << endl;
	ss << "Parent: " << (void*) parent << endl;
	ss << "Types:" << endl;
	for (std::pair<string, DataType*> type : types) {
		ss << "type " << type.first << ": " << type.second << endl;
	}
	ss << "Funcs:" << endl;
	for (std::pair<string, FuncScope*> func : funcs) {
		ss << "func " << func.first << ": " << func.second << endl;
	}
	ss << "Vars:" << endl;
	for (std::pair<string, Variable*> var : vars) {
		ss << "var " << var.first << ": " << var.second << endl;
	}
	for (int i = 0; i < tempVars.size(); i++) {
		ss << "var !TEMP" << (i + 1) << ": " << tempVars[i] << endl;
	}
	ss << "Commands:" << endl;
	for (Command* com : commands) {
		ss << com->repr() << endl;
	}
	for (FuncScope *tfunc : tempFuncs) {
		ss << tfunc->repr();
	}
	for (std::pair<string, FuncScope*> func : funcs) {
		// Ignore builtin functions with no code.
		if (func.second->commands.size() > 0)
			ss << func.second->repr();
	}
	return ss.str();
}

string FuncScope::repr() {
	stringstream ss;
	ss << "=== FUNC SCOPE ===\n";
	ss << "Inputs: ";
	for (Variable *input : ins) {
		ss << (void*) input << ", ";
	}
	ss << "\nOutputs: ";
	for (Variable *output: outs) {
		ss << (void*) output << ", ";
	}
	ss << "\n" << Scope::repr();
	return ss.str();
}

void parseStatList(Scope *scope, StatList *slist) {
	for (Statement *stat : slist->getStatements()) { 
		stat->convert(scope);
	}
	// Postpone parsing the body of functions until everything has been defined.
	// (Function hoisting)
	for (Statement *stat : slist->getStatements()) {
		if (FuncDec *fdec = dynamic_cast<FuncDec*>(stat)) {
			parseStatList(scope->lookupFunc(fdec->getName()), fdec->getBody());
		}
	}
	cout << scope->repr() << endl;
}

void parseSyntaxTree(StatList* slist) {
	Scope *root = new Scope();
	BUILTIN_ADD = new FuncScope(root);
	BUILTIN_MUL = new FuncScope(root);
	BUILTIN_RECIP = new FuncScope(root);
	BUILTIN_ITOF = new FuncScope(root);
	BUILTIN_BTOF = new FuncScope(root);
	BUILTIN_BTOI = new FuncScope(root);
	BUILTIN_ITOB = new FuncScope(root);
	BUILTIN_FTOI = new FuncScope(root);
	BUILTIN_FTOB = new FuncScope(root);
	BUILTIN_COPY = new FuncScope(root);
	BUILTIN_MOD = new FuncScope(root);
	BUILTIN_EQ = new FuncScope(root);
	BUILTIN_NEQ = new FuncScope(root);
	BUILTIN_LTE = new FuncScope(root);
	BUILTIN_GTE = new FuncScope(root);
	BUILTIN_LT = new FuncScope(root);
	BUILTIN_GT = new FuncScope(root);
	BUILTIN_AND = new FuncScope(root);
	BUILTIN_OR = new FuncScope(root);
	BUILTIN_XOR = new FuncScope(root);
	BUILTIN_BAND = new FuncScope(root);
	BUILTIN_BOR = new FuncScope(root);
	BUILTIN_BXOR = new FuncScope(root);
	BUILTIN_INDEX = new FuncScope(root);
	root->declareFunc("!ADD", BUILTIN_ADD);
	root->declareFunc("!MUL", BUILTIN_MUL);
	root->declareFunc("!RECIP", BUILTIN_RECIP);
	root->declareFunc("!ITOF", BUILTIN_ITOF);
	root->declareFunc("!BTOF", BUILTIN_BTOF);
	root->declareFunc("!BTOI", BUILTIN_BTOI);
	root->declareFunc("!ITOB", BUILTIN_ITOB);
	root->declareFunc("!FTOI", BUILTIN_FTOI);
	root->declareFunc("!FTOB", BUILTIN_FTOB);
	root->declareFunc("!COPY", BUILTIN_COPY);
	root->declareFunc("!MOD", BUILTIN_MOD);
	root->declareFunc("!EQ", BUILTIN_EQ);
	root->declareFunc("!NEQ", BUILTIN_NEQ);
	root->declareFunc("!LTE", BUILTIN_LTE);
	root->declareFunc("!GTE", BUILTIN_GTE);
	root->declareFunc("!LT", BUILTIN_LT);
	root->declareFunc("!GT", BUILTIN_GT);
	root->declareFunc("!AND", BUILTIN_AND);
	root->declareFunc("!OR", BUILTIN_OR);
	root->declareFunc("!XOR", BUILTIN_XOR);
	root->declareFunc("!BAND", BUILTIN_BAND);
	root->declareFunc("!BOR", BUILTIN_BOR);
	root->declareFunc("!BXOR", BUILTIN_BXOR);
	root->declareFunc("!INDEX", BUILTIN_INDEX);
	DATA_TYPE_INT = new DataType(4);
	DATA_TYPE_FLOAT = new DataType(4);
	DATA_TYPE_BOOL = new DataType(1);
	root->declareType("Int", DATA_TYPE_INT);
	root->declareType("Float", DATA_TYPE_FLOAT);
	root->declareType("Bool", DATA_TYPE_BOOL);
	parseStatList(root, slist);
}

}
