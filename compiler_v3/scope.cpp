#include <iostream>
#include <sstream>
#include "scope.h"
#include "tokens.h"

namespace Com {

FuncScope *Scope::lookupFunc(string name) {
	if (funcs.count(name)) {
		return funcs[name];
	} else if (parent != nullptr) {
		return parent->lookupFunc(name);
	} else {
		return nullptr;
	}
}

Value *Scope::lookupVar(string name) {
	if (vars.count(name)) {
		return vars[name];
	} else if (parent != nullptr) {
		return parent->lookupVar(name);
	} else {
		return nullptr;
	}
}

DataType *Scope::lookupType(string name) {
	if (types.count(name)) {
		return types[name];
	} else if (parent != nullptr) {
		return parent->lookupType(name);
	} else {
		return nullptr;
	}
}

int getDataTypeIndex(DataType *t) {
	if (t == nullptr) return 0;
	if (t == DATA_TYPE_BOOL) return 10;
	if (t == DATA_TYPE_INT) return 20;
	if (t == DATA_TYPE_FLOAT) return 30;
	if (t->getArrayDepth() > 0) // Guaranteed to be larger than a smaller depth array type.
		return getDataTypeIndex(t->getLowestType()) * 10;
	return 0;
}

DataType *pickBiggerType(DataType *a, DataType *b) {
	int aindex = getDataTypeIndex(a), bindex = getDataTypeIndex(b);
	return (aindex > bindex) ? a : b;
}

FuncScope *builtinCastFunc(DataType *from, DataType *to) {
	if (from == DATA_TYPE_BOOL) {
		if (to == DATA_TYPE_INT)
			return BUILTIN_BTOI;
		else if (to == DATA_TYPE_FLOAT)
			return BUILTIN_BTOF;
	} else if (from == DATA_TYPE_INT) {
		if (to == DATA_TYPE_BOOL)
			return BUILTIN_ITOB;
		else if (to == DATA_TYPE_FLOAT) 
			return BUILTIN_ITOF;
	} else if (from == DATA_TYPE_FLOAT) {
		if (to == DATA_TYPE_INT)
			return BUILTIN_FTOI;
		else if (to == DATA_TYPE_BOOL)
			return BUILTIN_FTOB;
	} 
	if (from == to) {
		return BUILTIN_COPY;
	}
	return (FuncScope*) 0xDEADBEEF;
}

void Scope::castValue(Value *from, Value *to) {
	DataType *tfrom = from->getType(), *tto = to->getType();
	FuncScope *fs = (FuncScope*) 0xDEADBEEF;
	if (tfrom->getArrayDepth() == 0 && tto->getArrayDepth()) {
		fs = builtinCastFunc(tfrom, tto);
	} else {
		if (tto->getArrayDepth() == tfrom->getArrayDepth()) {
			fs = builtinCastFunc(tfrom->getLowestType(), tto->getLowestType());
		}
	}
	// TODO: Complex array casts.
	// Int[1] -> Int[5] should copy 5 times
	// Int[2] -> Int[5] should make [a[0], a[1], a[0], a[1], a[0]]
	// Int[3] -> Int[5] should make [a[0], a[1], a[2], a[0], a[1]]
	// Int[5] -> Int[1] should throw an error
	// Int[1] -> Bool[5] should convert and copy 5 times
	// Int    -> Int[5] should be treated as Int[1] -> Int[5]
	// Int[8] -> Int[5][8] should be treated as Int -> Int[5] for each of 8 elements.
	// Int[5][1] -> Int[5][8] should be treated as Int[5] -> Int[5] for each of 8 elements.
	// Int[5][99] -> Int[5][8] should throw an error, just like Int[99] -> Int[8]
	Command *cc = new Command(fs);
	cc->addInput(from);
	cc->addOutput(to);
	commands.push_back(cc);
}

void Scope::addCommand(Command *command) { 
	std::vector<int> upcastIns, upcastOuts;
	std::vector<Value*> &ins = command->getFuncScope()->getIns(),
		&outs = command->getFuncScope()->getOuts();
	int i = 0;
	for (auto input : command->getFuncScope()->getIns()) {
		if (input->getType() == UPCAST_WILDCARD) {
			upcastIns.push_back(i);
		}
		i++;
	}
	i = 0;
	for (auto output : command->getFuncScope()->getOuts()) {
		if (output->getType() == UPCAST_WILDCARD) {
			upcastOuts.push_back(i);
		}
		i++;
	}
	if (upcastIns.size() > 0) { 
		DataType *biggest = command->getIns()[upcastIns[0]]->getType();
		for (int i = 1; i < upcastIns.size(); i++) {
			biggest = pickBiggerType(biggest, command->getIns()[upcastIns[i]]->getType());
		}
		for (int index : upcastIns) {
			DataType *dtut = command->getIns()[index]->getType();
			if (dtut->getArrayDepth() != biggest->getArrayDepth() 
					|| dtut->getLowestType() != biggest->getLowestType()) {
				Value *tvar = new Value(biggest);
				declareTempVar(tvar);
				castValue(command->getIns()[index], tvar);
				command->getIns()[index] = tvar;
			}
		}
		commands.push_back(command); 
		for (int index : upcastOuts) {
			Value *vut = command->getOuts()[index];
			if (vut->getType() == UPCAST_WILDCARD) { // Only temp vars can be that type.
				vut->setType(biggest);
			} else if (vut->getType() != biggest) {
				Value *tvar = new Value(biggest);
				declareTempVar(tvar);
				command->getOuts()[index] = tvar;
				castValue(tvar, vut);
			}
		}
	} else {
		commands.push_back(command); 
	}
}

void FuncScope::declareVar(string name, Value *variable) {
	Scope::declareVar(name, variable);
	if (autoAdd == 1) {
		ins.push_back(variable);
	} else if (autoAdd == 2) {
		outs.push_back(variable);
	}
}

Value::Value(DataType *type)
	: type(type) {
	data = malloc(type->getLength());
}

string Value::repr(DataType *dtype, void *data) { 
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

string Value::repr() {
	if (constant) {
		return repr(type, data);
	} else {
		stringstream ss;
		ss << "var " << (void*) type << " " << this;
		return ss.str();
	}
}

void Value::setType(DataType *newType) {
	type = newType;
	if (data != nullptr) {
		free(data);
		data = malloc(type->getLength());
	}
}

vector<Command*> &Command::getCommands() { 
	return call->getCommands(); 
}

string Command::repr() {
	stringstream ss;
	ss << "funcc " << (void*) call;
	for (Value *input : ins) {
		ss << " " << input->repr();
	}
	ss << ",";
	for (Value *output : outs) {
		ss << " " << output->repr();
	}
	if (aug != nullptr) {
		ss << " AUG ";
		switch (aug->getType()) {
		case AugmentationType::DO_IF:
			ss << "DO_IF ";
			break;
		case AugmentationType::DO_IF_NOT:
			ss << "DO_IF_NOT ";
			break;
		case AugmentationType::LOOP_FOR:
			ss << "LOOP_FOR ";
			break;
		case AugmentationType::LOOP_RANGE:
			ss << "LOOP_RANGE ";
			break;
		}
		for (auto param : aug->getParams()) {
			ss << param->repr() << " ";
		}
	}
	return ss.str();
}

DataType *DATA_TYPE_INT, *DATA_TYPE_FLOAT, *DATA_TYPE_BOOL, *UPCAST_WILDCARD, *ANY_WILDCARD;
FuncScope *BUILTIN_ADD, *BUILTIN_MUL, *BUILTIN_RECIP, *BUILTIN_MOD;
FuncScope *BUILTIN_ITOF, *BUILTIN_BTOF, *BUILTIN_BTOI, *BUILTIN_ITOB, *BUILTIN_FTOI, *BUILTIN_FTOB;
FuncScope *BUILTIN_EQ, *BUILTIN_NEQ, *BUILTIN_LTE, *BUILTIN_GTE, *BUILTIN_LT, *BUILTIN_GT;
FuncScope *BUILTIN_AND, *BUILTIN_OR, *BUILTIN_XOR, *BUILTIN_BAND, *BUILTIN_BOR, *BUILTIN_BXOR;
FuncScope *BUILTIN_COPY;

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
	for (std::pair<string, Value*> var : vars) {
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
	for (Value *input : ins) {
		ss << (void*) input << ", ";
	}
	ss << "\nOutputs: ";
	for (Value *output: outs) {
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
}

Scope *parseSyntaxTree(StatList* slist) {
	Scope *root = new Scope();

	DATA_TYPE_INT = new DataType(4);
	DATA_TYPE_FLOAT = new DataType(4);
	DATA_TYPE_BOOL = new DataType(1);
	UPCAST_WILDCARD = new DataType(1);
	ANY_WILDCARD = new DataType(1);
	root->declareType("Int", DATA_TYPE_INT);
	root->declareType("Float", DATA_TYPE_FLOAT);
	root->declareType("Bool", DATA_TYPE_BOOL);
	root->declareType("!UPCAST_WILDCARD", UPCAST_WILDCARD);
	root->declareType("!ANY_WILDCARD", ANY_WILDCARD);

	BUILTIN_ADD = new FuncScope(root);
	BUILTIN_ADD->autoAddIns();
	BUILTIN_ADD->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_ADD->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_ADD->autoAddOuts();
	BUILTIN_ADD->declareVar("x", new Value(UPCAST_WILDCARD));

	BUILTIN_MUL = new FuncScope(root);
	BUILTIN_MUL->autoAddIns();
	BUILTIN_MUL->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_MUL->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_MUL->autoAddOuts();
	BUILTIN_MUL->declareVar("x", new Value(UPCAST_WILDCARD));

	BUILTIN_RECIP = new FuncScope(root);
	BUILTIN_RECIP->autoAddIns();
	BUILTIN_RECIP->declareVar("a", new Value(DATA_TYPE_FLOAT));
	BUILTIN_RECIP->autoAddOuts();
	BUILTIN_RECIP->declareVar("x", new Value(DATA_TYPE_FLOAT));

	BUILTIN_ITOF = new FuncScope(root);
	BUILTIN_ITOF->autoAddIns();
	BUILTIN_ITOF->declareVar("a", new Value(DATA_TYPE_INT));
	BUILTIN_ITOF->autoAddOuts();
	BUILTIN_ITOF->declareVar("x", new Value(DATA_TYPE_FLOAT));

	BUILTIN_BTOF = new FuncScope(root);
	BUILTIN_BTOF->autoAddIns();
	BUILTIN_BTOF->declareVar("a", new Value(DATA_TYPE_BOOL));
	BUILTIN_BTOF->autoAddOuts();
	BUILTIN_BTOF->declareVar("x", new Value(DATA_TYPE_FLOAT));

	BUILTIN_BTOI = new FuncScope(root);
	BUILTIN_BTOI->autoAddIns();
	BUILTIN_BTOI->declareVar("a", new Value(DATA_TYPE_BOOL));
	BUILTIN_BTOI->autoAddOuts();
	BUILTIN_BTOI->declareVar("x", new Value(DATA_TYPE_INT));

	BUILTIN_ITOB = new FuncScope(root);
	BUILTIN_ITOB->autoAddIns();
	BUILTIN_ITOB->declareVar("a", new Value(DATA_TYPE_INT));
	BUILTIN_ITOB->autoAddOuts();
	BUILTIN_ITOB->declareVar("x", new Value(DATA_TYPE_BOOL));

	BUILTIN_FTOI = new FuncScope(root);
	BUILTIN_FTOI->autoAddIns();
	BUILTIN_FTOI->declareVar("a", new Value(DATA_TYPE_FLOAT));
	BUILTIN_FTOI->autoAddOuts();
	BUILTIN_FTOI->declareVar("x", new Value(DATA_TYPE_INT));

	BUILTIN_FTOB = new FuncScope(root);
	BUILTIN_FTOB->autoAddIns();
	BUILTIN_FTOB->declareVar("a", new Value(DATA_TYPE_FLOAT));
	BUILTIN_FTOB->autoAddOuts();
	BUILTIN_FTOB->declareVar("x", new Value(DATA_TYPE_BOOL));

	// The way this one works is a bit weird. If the input and output are the same size, OFFSET should be zero. The
	// entire value will be copied. If one is bigger than the other, a chunk of data the size of the smaller one will
	// be transferred. OFFSET will be used as the byte index to start copying from from the larger data type.
	BUILTIN_COPY = new FuncScope(root);
	BUILTIN_COPY->autoAddIns();
	BUILTIN_COPY->declareVar("a", new Value(ANY_WILDCARD));
	BUILTIN_COPY->declareVar("offset", new Value(DATA_TYPE_INT));
	BUILTIN_COPY->autoAddOuts();
	BUILTIN_COPY->declareVar("x", new Value(ANY_WILDCARD));

	BUILTIN_MOD = new FuncScope(root);
	BUILTIN_MOD->autoAddIns();
	BUILTIN_MOD->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_MOD->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_MOD->autoAddOuts();
	BUILTIN_MOD->declareVar("x", new Value(UPCAST_WILDCARD));

	BUILTIN_EQ = new FuncScope(root);
	BUILTIN_EQ->autoAddIns();
	BUILTIN_EQ->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_EQ->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_EQ->autoAddOuts();
	BUILTIN_EQ->declareVar("x", new Value(DATA_TYPE_BOOL));

	BUILTIN_NEQ = new FuncScope(root);
	BUILTIN_NEQ->autoAddIns();
	BUILTIN_NEQ->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_NEQ->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_NEQ->autoAddOuts();
	BUILTIN_NEQ->declareVar("x", new Value(DATA_TYPE_BOOL));

	BUILTIN_LTE = new FuncScope(root);
	BUILTIN_LTE->autoAddIns();
	BUILTIN_LTE->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_LTE->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_LTE->autoAddOuts();
	BUILTIN_LTE->declareVar("x", new Value(DATA_TYPE_BOOL));

	BUILTIN_GTE = new FuncScope(root);
	BUILTIN_GTE->autoAddIns();
	BUILTIN_GTE->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_GTE->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_GTE->autoAddOuts();
	BUILTIN_GTE->declareVar("x", new Value(DATA_TYPE_BOOL));

	BUILTIN_LT = new FuncScope(root);
	BUILTIN_LT->autoAddIns();
	BUILTIN_LT->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_LT->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_LT->autoAddOuts();
	BUILTIN_LT->declareVar("x", new Value(DATA_TYPE_BOOL));

	BUILTIN_GT = new FuncScope(root);
	BUILTIN_GT->autoAddIns();
	BUILTIN_GT->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_GT->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_GT->autoAddOuts();
	BUILTIN_GT->declareVar("x", new Value(DATA_TYPE_BOOL));

	BUILTIN_AND = new FuncScope(root);
	BUILTIN_AND->autoAddIns();
	BUILTIN_AND->declareVar("a", new Value(DATA_TYPE_BOOL));
	BUILTIN_AND->declareVar("b", new Value(DATA_TYPE_BOOL));
	BUILTIN_AND->autoAddOuts();
	BUILTIN_AND->declareVar("x", new Value(DATA_TYPE_BOOL));

	BUILTIN_OR = new FuncScope(root);
	BUILTIN_OR->autoAddIns();
	BUILTIN_OR->declareVar("a", new Value(DATA_TYPE_BOOL));
	BUILTIN_OR->declareVar("b", new Value(DATA_TYPE_BOOL));
	BUILTIN_OR->autoAddOuts();
	BUILTIN_OR->declareVar("x", new Value(DATA_TYPE_BOOL));

	BUILTIN_XOR = new FuncScope(root);
	BUILTIN_XOR->autoAddIns();
	BUILTIN_XOR->declareVar("a", new Value(DATA_TYPE_BOOL));
	BUILTIN_XOR->declareVar("b", new Value(DATA_TYPE_BOOL));
	BUILTIN_XOR->autoAddOuts();
	BUILTIN_XOR->declareVar("x", new Value(DATA_TYPE_BOOL));

	BUILTIN_BAND = new FuncScope(root);
	BUILTIN_BAND->autoAddIns();
	BUILTIN_BAND->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_BAND->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_BAND->autoAddOuts();
	BUILTIN_BAND->declareVar("x", new Value(UPCAST_WILDCARD));

	BUILTIN_BOR = new FuncScope(root);
	BUILTIN_BOR->autoAddIns();
	BUILTIN_BOR->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_BOR->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_BOR->autoAddOuts();
	BUILTIN_BOR->declareVar("x", new Value(UPCAST_WILDCARD));

	BUILTIN_BXOR = new FuncScope(root);
	BUILTIN_BXOR->autoAddIns();
	BUILTIN_BXOR->declareVar("a", new Value(UPCAST_WILDCARD));
	BUILTIN_BXOR->declareVar("b", new Value(UPCAST_WILDCARD));
	BUILTIN_BXOR->autoAddOuts();
	BUILTIN_BXOR->declareVar("x", new Value(UPCAST_WILDCARD));

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

	parseStatList(root, slist);
	return root;
}

}
