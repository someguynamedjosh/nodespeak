#ifndef _TOKENS_H_
#define _TOKENS_H_

#include <string>
#include <vector>
using namespace std;

class Token {
public:
	virtual string repr() { return "Token"; }
};
class Root: public Token {};
class Expression: public Token {
public:
	virtual string repr() { return "Exp"; }
};
class IdentifierExp: public Expression {
protected:
	string name;
public:
	IdentifierExp(string name): name(name) {}
	string repr() { return "[Identifier " + string(name) + "]"; }
};
class IntExp: public Expression {
protected:
	int value;
public:
	IntExp(int value): value(value) {}
	string repr() { return to_string(value); }
};
class FloatExp: public Expression {
protected:
	float value;
public:
	FloatExp(float value) : value(value) {}
	string repr() { return to_string(value); }
};
class OperatorExp: public Expression {
protected:
	vector<Expression*> args;
	string reprHelp(string op) { 
		string tr = "[" + op + "](";
		for(Expression* exp : args) {
			tr += " " + exp->repr();
		}
		return tr + ")";
	}
public:
	OperatorExp() {}
	void addArg(Expression *arg) { args.push_back(arg); }
	void addArgRec(Expression *arg) { 
		if(typeid(*arg) == typeid(*this)) {
			for(Expression *subArg : ((OperatorExp*) arg)->args) {
				args.push_back(subArg);
			}
		} else {
			args.push_back(arg);
		}
	}
};
class AddExp: public OperatorExp {
public:
	AddExp(Expression *a, Expression *b) { addArgRec(a); addArgRec(b); }
	string repr() { return reprHelp("add"); }
};
class IncExp: public OperatorExp {
public:
	IncExp(Expression *a) { addArg(a); }
	string repr() { return reprHelp("inc"); }
};
class DecExp: public OperatorExp {
public:
	DecExp(Expression *a) { addArg(a); }
	string repr() { return reprHelp("dec"); }
};
class MulExp: public OperatorExp {
public:
	MulExp(Expression *a, Expression *b) { addArgRec(a); addArgRec(b); }
	string repr() { return reprHelp("mul"); }
};
class RecipExp: public OperatorExp {
public:
	RecipExp(Expression *a) { addArg(a); }
	string repr() { return reprHelp("recip"); }
};
class ModExp: public OperatorExp {
public:
	ModExp(Expression *a, Expression *b) { addArg(a); addArg(b); }
	string repr() { return reprHelp("mod"); }
};
#define OP_EXP_HELP(CNAME, HNAME) class CNAME : public OperatorExp { \
public: \
	CNAME(Expression *a, Expression *b) { addArg(a); addArg(b); } \
	string repr() { return reprHelp(HNAME); } \
};
OP_EXP_HELP(EqExp, "eq")
OP_EXP_HELP(NeqExp, "neq")
OP_EXP_HELP(LteExp, "lte")
OP_EXP_HELP(GteExp, "gte")
OP_EXP_HELP(LtExp, "lt")
OP_EXP_HELP(GtExp, "gt")
OP_EXP_HELP(AndExp, "and")
OP_EXP_HELP(OrExp, "or")
OP_EXP_HELP(XorExp, "xor")
OP_EXP_HELP(BandExp, "band")
OP_EXP_HELP(BorExp, "bor")
OP_EXP_HELP(BxorExp, "bxor")
class ArrayAccessExp: public Expression {
protected:
	Expression *from, *index;
public:
	ArrayAccessExp(Expression *from, Expression *index): from(from), index(index) { }
	string repr() { return from->repr() + "[" + index->repr() + "]"; }
};
class MemberAccessExp: public Expression {
protected:
	Expression *from;
	string name;
public:
	MemberAccessExp(Expression *from, string memberName): from(from), name(memberName) { }
	string repr() { return from->repr() + "." + name; }
};
class ExpList: public Token {
protected:
	vector<Expression*> exps;
public:
	ExpList(Expression *a) { append(a); }
	ExpList(ExpList *a, Expression *b) { append(a); append(b); }
	ExpList(ExpList *a, ExpList *b) { append(a); append(b); }
	void append(Expression *a) { exps.push_back(a); }
	void append(ExpList *a) { for(Expression *e : a->exps) append(e); }
	string repr() {
		string tr = "";
		for (Expression *e : exps) {
			tr += e->repr() + ", ";
		}
		return tr;
	}
};
class ArrayLiteral: public Expression {
protected:
	ExpList *elements;
public:
	ArrayLiteral(ExpList *elements): elements(elements) { }
	string repr() { return "Array [" + elements->repr() + "]"; }
};
class Range: public Expression {
protected:
	Expression *start, *end, *step = 0;
public:
	Range(Expression *start, Expression *end): start(start), end(end) { }
	Range(Expression *start, Expression *end, Expression *step): start(start), end(end), step(step) { }
	string repr() { return "{" + start->repr() + ", " + end->repr() + ((step == 0) ? "" : ", " + step->repr()) + "}"; }
};
class Output: public Token { };
class RetOut: public Output { // Only used for return statements in function calls.
public:
	RetOut() { }
	string repr() { return "return"; }
};
class NoneOut: public Output { // Only used for none statements in function calls.
public:
	NoneOut() { }
	string repr() { return "none"; }
};
class VarAccessOut: public Output {
protected:
	Expression *accessExp;
public:
	VarAccessOut(Expression *accessExp): accessExp(accessExp) { }
	string repr() { return accessExp->repr(); }
};
class OutList: public Token {
protected:
	vector<Output*> outputs;
public:
	OutList(Output *a) { append(a); }
	OutList(OutList *a, OutList *b) { append(a); append(b); }
	void append(Output *a) { outputs.push_back(a); }
	void append(OutList *a) { for(Output *o : a->outputs) outputs.push_back(o); }
	string repr() {
		string tr = "";
		for(Output *o : outputs) {
			tr += o->repr() + ", ";
		}
		return tr;
	}
};
class FuncCall: public Expression {
protected:
	string name;
	ExpList *ins;
	OutList *outs;
public:
	FuncCall(string name, ExpList *ins, OutList *outs): name(name), ins(ins), outs(outs) { }
	string repr() { return name + "(" + ins->repr() + "):(" + outs->repr() + ")"; }
};

class Statement: public Token {};
class FuncCallStat: public Statement {
protected:
	FuncCall *call;
public:
	FuncCallStat(FuncCall *call): call(call) { }
	string repr() { return call->repr(); }
};
class AssignStat: public Statement {
protected:
	Expression *value, *to;
public:
	AssignStat(Expression *to, Expression *value): value(value), to(to) { }
	string repr() { return to->repr() + " = " + value->repr(); }
};
class ReturnStat: public Statement { };

class StatList: public Token {
protected:
	vector<Statement*> stats;
public:
	StatList() { }
	StatList(Statement *a) { stats.push_back(a); }
	StatList(StatList *a) { append(a); }
	StatList(Statement *a, Statement *b) { stats.push_back(a); stats.push_back(b); }
	StatList(StatList *a, Statement *b) { append(a); append(b); }
	StatList(StatList *a, StatList *b) { append(a); append(b); }
	void append(Statement *a) { stats.push_back(a); }
	void append(StatList *a) { for(Statement *s : a->stats) append(s); }
	string repr() {
		string tr = "";
		for(Statement *s : stats) {
			tr += s->repr() + "\n";
		}
		return tr;
	}
	vector<Statement*>& getStatements() { return stats; }
};
class FuncDec: public Statement {
protected:
	string name;
	StatList *ins, *outs, *code;
public:
	FuncDec(string name, StatList *ins, StatList *outs, StatList *code): name(name), ins(ins), outs(outs), code(code) { }
	string repr() { return "Declare func: " + name + "(\n" + ins->repr() + "):(\n" + outs->repr() + ") {\n" + code->repr() + "}"; }
};
class Branch: public Statement {
protected:
	Expression *con;
	StatList *ifTrue, *ifFalse = 0;
	Branch *elseClause = 0;
public:
	Branch(Expression *con, StatList *ifTrue): con(con), ifTrue(ifTrue) { }
	void addElse(StatList *contents) {
		if(elseClause == 0) ifFalse = contents; else elseClause->addElse(contents);
	}
	void addElif(Branch *branch) {
		if(elseClause == 0) { ifFalse = new StatList(branch); elseClause = branch; } else elseClause->addElif(branch);
	}
	string repr() { return "Branch on " + con->repr() + "\nIf true:\n" + ifTrue->repr() + ((ifFalse == 0) ? "" : "If false:\n" + ifFalse->repr()); }
};

class Type: public Token { };
class TypeName: public Type {
protected:
	string name;
public: 
	TypeName(string name): name(name) { }
	string repr() { return name; }
};
class ArrayType: public Type {
protected:
	Type *baseType;
	Expression *size;
public:
	ArrayType(Type *baseType, Expression *size): baseType(baseType), size(size) { }
	string repr() { return baseType->repr() + "[" + size->repr() + "]"; }
};
class VarDec: public Statement {
protected:
	Type *type;
	string name;
public:
	VarDec(Type *type, string name): type(type), name(name) { }
	string repr() { return "Declare var: " + type->repr() + " named " + name; }
	Type* getType() { return type; }
};
class ForLoop: public Statement {
protected:
	VarDec *counter;
	ExpList *values;
	StatList *body;
public:
	ForLoop(VarDec *counter, ExpList *values, StatList *body): counter(counter), values(values), body(body) { }
};
class WhileLoop: public Statement {
protected:
	Expression *condition;
	StatList *body;
public:
	WhileLoop(Expression *condition, StatList *body): condition(condition), body(body) { }
};

#endif
