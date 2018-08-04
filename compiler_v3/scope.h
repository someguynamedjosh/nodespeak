#ifndef _SCOPE_H
#define _SCOPE_H

#include <map>
#include <string>
#include <vector>
using namespace std;

class StatList;

namespace Com {

class Scope;
class FuncScope;

class DataType { 
private:
	int length;
public:
	DataType(int length): length(length) { }
	int getLength() { return length; }
	virtual int getArrayDepth() { return 0; }
	virtual DataType *getBaseType() { return this; }
};
extern DataType *DATA_TYPE_INT, *DATA_TYPE_FLOAT, *DATA_TYPE_BOOL, *UPCAST_WILDCARD;

class ElementaryDataType: public DataType { 
public:
	ElementaryDataType(int length): DataType(length) { }
};

class ArrayDataType: public DataType {
private:
	DataType *baseType;
public:
	ArrayDataType(DataType *baseType, int length): DataType(baseType->getLength() * length), baseType(baseType) { }
	virtual int getArrayDepth() { return baseType->getArrayDepth() + 1; }
	virtual int getArrayLength() { return getLength() / baseType->getLength(); }
	virtual DataType *getBaseType() { return baseType; }
};

class Literal {
private:
	DataType *type;
	void *data;
public:
	Literal(DataType *type);
	Literal(DataType *type, void *data): type(type), data(data) { }
	DataType *getType() { return type; }
	void *getData() { return data; }
	float *interpretAsFloat() { return static_cast<float*>(data); }
	int *interpretAsInt() { return static_cast<int*>(data); }
	bool *interpretAsBool() { return static_cast<bool*>(data); }
	virtual string repr(DataType *dtype, void *data);
	virtual string repr();
};

class Variable { 
private:
	DataType *type;
	Literal *currentValue = nullptr;
public:
	Variable(DataType *type): type(type) { }
	void setType(DataType *type) { this->type = type; }
	DataType *getType() { return type; }
	virtual string repr();
	void initCurrentValue() { currentValue = new Literal(type); }
	Literal *getCurrentValue() { return currentValue; }
};

class FuncInput {
private:
	Variable *varIn;
	Literal *litIn;
public:
	FuncInput(Variable *variable): varIn(variable), litIn(nullptr) { }
	FuncInput(Literal *literal): varIn(nullptr), litIn(literal) { }
	bool isVariable() { return litIn == nullptr; }
	bool isLiteral() { return varIn == nullptr; }
	Variable *getVariable() { return varIn; }
	Literal *getLiteral() { return litIn; }
	DataType *getType() { return (varIn) ? varIn->getType() : litIn->getType(); }
	Literal *getCurrentValue() { return (varIn) ? varIn->getCurrentValue() : litIn; }
	virtual string repr();
};

class Command {
private:
	vector<FuncInput*> ins;
	vector<Variable*> outs;
	FuncScope *call;
public:
	Command(FuncScope *call): call(call) { }
	void addInput(FuncInput *input) { ins.push_back(input); }
	void addOutput(Variable *output) { outs.push_back(output); }
	vector<FuncInput*> &getIns() { return ins; }
	vector<Variable*> &getOuts() { return outs; }
	FuncScope *getFuncScope() { return call; }
	vector<Command*> &getCommands();
	virtual string repr();
};

class Scope {
private:
	map<string, FuncScope*> funcs;
	vector<FuncScope*> tempFuncs;
	map<string, Variable*> vars;
	vector<Variable*> tempVars;
	map<string, DataType*> types;
	vector<Command*> commands;
	Scope *parent;
	void castValue(FuncInput *from, Variable *to);
public:
	Scope(): parent(nullptr) { };
	Scope(Scope *parent): parent(parent) { }
	Scope *getParent() { return parent; }
	FuncScope *lookupFunc(string name);
	Variable *lookupVar(string name);
	DataType *lookupType(string name);
	void declareFunc(string name, FuncScope *func) { funcs.emplace(name, func); }
	void declareTempFunc(FuncScope *func) { tempFuncs.push_back(func); }
	virtual void declareVar(string name, Variable *variable) { vars.emplace(name, variable); }
	void declareTempVar(Variable *variable) { tempVars.push_back(variable); }
	void declareType(string name, DataType *type) { types.emplace(name, type); }
	void addCommand(Command *command);
	vector<Command*> &getCommands() { return commands; }
	virtual string repr();
};

class FuncScope: public Scope { 
private:
	vector<Variable*> ins, outs;
	int autoAdd = 0;
public:
	FuncScope(Scope *parent): Scope(parent) { }
	vector<Variable*>& getIns() { return ins; }
	vector<Variable*>& getOuts() { return outs; }
	void addIn(Variable* _in) { ins.push_back(_in); }
	void addOut(Variable* out) { outs.push_back(out); }
	void autoAddNone() { autoAdd = 0; }
	void autoAddIns() { autoAdd = 1; }
	void autoAddOuts() { autoAdd = 2; }
	virtual void declareVar(string name, Variable *variable);
	virtual string repr();
};

extern FuncScope *BUILTIN_ADD, *BUILTIN_MUL, *BUILTIN_RECIP, *BUILTIN_MOD;
extern FuncScope *BUILTIN_ITOF, *BUILTIN_BTOF, *BUILTIN_BTOI, *BUILTIN_ITOB, *BUILTIN_FTOI, *BUILTIN_FTOB;
extern FuncScope *BUILTIN_COPY;
extern FuncScope *BUILTIN_EQ, *BUILTIN_NEQ, *BUILTIN_LTE, *BUILTIN_GTE, *BUILTIN_LT, *BUILTIN_GT;
extern FuncScope *BUILTIN_AND, *BUILTIN_OR, *BUILTIN_XOR, *BUILTIN_BAND, *BUILTIN_BOR, *BUILTIN_BXOR;
extern FuncScope *BUILTIN_INDEX;

Literal *evalBuiltinFunc(FuncScope *func, Literal *a, Literal *b);

void parseSyntaxTree(StatList* list);

}

#endif
