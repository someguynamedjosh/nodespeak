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
	virtual DataType *getLowestType() { return this; }
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
	DataType *getBaseType() { return baseType; }
	virtual DataType *getLowestType() { return baseType->getLowestType(); }
};

class Value {
private:
	DataType *type;
	void *data;
	bool constant = false;
public:
	Value(DataType *type);
	Value(DataType *type, void *data): type(type), data(data), constant(true) { }
	DataType *getType() { return type; }
	void *getData() { return data; }
	bool isConstant() { return constant; }
	float *interpretAsFloat() { return static_cast<float*>(data); }
	int *interpretAsInt() { return static_cast<int*>(data); }
	bool *interpretAsBool() { return static_cast<bool*>(data); }
	void setType(DataType *newType);
	virtual string repr(DataType *dtype, void *data);
	virtual string repr();
};

enum AugmentationType {
	DO_IF, DO_IF_NOT, LOOP_FOR, LOOP_RANGE
};

class Augmentation {
private:
	AugmentationType type;
	vector<Value*> params;
public:
	Augmentation(AugmentationType type): type(type) { }
	Augmentation(AugmentationType type, Value *param1): type(type) { params.push_back(param1); }
	Augmentation(AugmentationType type, Value *param1, Value *param2): type(type) { 
		params.push_back(param1); params.push_back(param2); }
	AugmentationType getType() { return type; }
	vector<Value*>& getParams() { return params; }
};

class Command {
private:
	vector<Value*> ins;
	vector<Value*> outs;
	Augmentation *aug = nullptr;
	FuncScope *call;
public:
	Command(FuncScope *call): call(call) { }
	Command(FuncScope *call, Augmentation *aug): call(call), aug(aug) { }
	void addInput(Value *input) { ins.push_back(input); }
	void addOutput(Value *output) { outs.push_back(output); }
	vector<Value*> &getIns() { return ins; }
	vector<Value*> &getOuts() { return outs; }
	Augmentation *getAugmentation() { return aug; }
	FuncScope *getFuncScope() { return call; }
	vector<Command*> &getCommands();
	virtual string repr();
};

class Scope {
private:
	map<string, FuncScope*> funcs;
	vector<FuncScope*> tempFuncs;
	map<string, Value*> vars;
	vector<Value*> tempVars;
	map<string, DataType*> types;
	vector<Command*> commands;
	Scope *parent;
	void castValue(Value *from, Value *to);
public:
	Scope(): parent(nullptr) { };
	Scope(Scope *parent): parent(parent) { }
	Scope *getParent() { return parent; }
	FuncScope *lookupFunc(string name);
	Value *lookupVar(string name);
	DataType *lookupType(string name);
	void declareFunc(string name, FuncScope *func) { funcs.emplace(name, func); }
	void declareTempFunc(FuncScope *func) { tempFuncs.push_back(func); }
	virtual void declareVar(string name, Value *variable) { vars.emplace(name, variable); }
	void declareTempVar(Value *variable) { tempVars.push_back(variable); }
	void declareType(string name, DataType *type) { types.emplace(name, type); }
	void addCommand(Command *command);
	vector<Command*> &getCommands() { return commands; }
	virtual string repr();
};

class FuncScope: public Scope { 
private:
	vector<Value*> ins, outs;
	int autoAdd = 0;
public:
	FuncScope(Scope *parent): Scope(parent) { }
	vector<Value*>& getIns() { return ins; }
	vector<Value*>& getOuts() { return outs; }
	void addIn(Value* _in) { ins.push_back(_in); }
	void addOut(Value* out) { outs.push_back(out); }
	void autoAddNone() { autoAdd = 0; }
	void autoAddIns() { autoAdd = 1; }
	void autoAddOuts() { autoAdd = 2; }
	virtual void declareVar(string name, Value *variable);
	virtual string repr();
};

extern FuncScope *BUILTIN_ADD, *BUILTIN_MUL, *BUILTIN_RECIP, *BUILTIN_MOD;
extern FuncScope *BUILTIN_ITOF, *BUILTIN_BTOF, *BUILTIN_BTOI, *BUILTIN_ITOB, *BUILTIN_FTOI, *BUILTIN_FTOB;
extern FuncScope *BUILTIN_COPY;
extern FuncScope *BUILTIN_EQ, *BUILTIN_NEQ, *BUILTIN_LTE, *BUILTIN_GTE, *BUILTIN_LT, *BUILTIN_GT;
extern FuncScope *BUILTIN_AND, *BUILTIN_OR, *BUILTIN_XOR, *BUILTIN_BAND, *BUILTIN_BOR, *BUILTIN_BXOR;
extern FuncScope *BUILTIN_INDEX;

Scope *parseSyntaxTree(StatList* list);

}

#endif
