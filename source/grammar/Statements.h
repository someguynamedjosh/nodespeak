#ifndef _WAVEGUIDE_GRAMMAR_STATEMENTS_H_
#define _WAVEGUIDE_GRAMMAR_STATEMENTS_H_

#include "Token.h"

namespace waveguide {
namespace grammar {

class AccessExp;
class DataType;
class ExpList;
class Expression;
class FuncCall;
class StatList;

class Statement: public Token {
public:
    virtual void convert(convert::ScopeSP context) = 0;
};

class FuncCallStat: public Statement {
private:
    std::shared_ptr<FuncCall> call;
public:
    FuncCallStat(std::shared_ptr<FuncCall> call);
    virtual void convert(convert::ScopeSP context);
};

class VarDec: public Statement {
private:
    std::shared_ptr<DataType> type;
    std::string name;
public:
    VarDec(std::shared_ptr<DataType> type, std::string name);
    std::shared_ptr<DataType> getType();
    std::string getName();
    virtual void convert(convert::ScopeSP context);
};

class AssignStat: public Statement {
private:
    std::shared_ptr<AccessExp> to;
    std::shared_ptr<Expression> value;
public:
    AssignStat(std::shared_ptr<AccessExp> to,
        std::shared_ptr<Expression> value);
    virtual void convert(convert::ScopeSP context);
};

class ReturnStat: public Statement { };

class FuncDec: public Statement {
private:
    std::string name;
    std::shared_ptr<StatList> ins, outs, body;
public:
    FuncDec(std::string name, std::shared_ptr<StatList> ins,
        std::shared_ptr<StatList> outs, std::shared_ptr<StatList> body);
    std::string getName();
    std::shared_ptr<StatList> getBody();
    virtual void convert(convert::ScopeSP context);
};

class Branch: public Statement {
private:
    std::shared_ptr<Expression> con;
    std::shared_ptr<StatList> ifTrue, ifFalse{nullptr};
    std::shared_ptr<Branch> elseClause{nullptr};
public:
    Branch(std::shared_ptr<Expression> con, std::shared_ptr<StatList> ifTrue);
    void addElse(std::shared_ptr<StatList> body);
    void addElif(std::shared_ptr<Branch> branch);
    virtual void convert(convert::ScopeSP context);
};

class ForLoop: public Statement {
private:
    std::shared_ptr<VarDec> counter;
    std::shared_ptr<ExpList> values;
    std::shared_ptr<StatList> body;
public:
    ForLoop(std::shared_ptr<VarDec> counter, std::shared_ptr<ExpList> values,
        std::shared_ptr<StatList> body);
    virtual void convert(convert::ScopeSP context);
};

class WhileLoop: public Statement {
private:
    std::shared_ptr<Expression> condition;
    std::shared_ptr<StatList> body;
public:
    WhileLoop(std::shared_ptr<Expression> condition,
        std::shared_ptr<StatList> body);
};

class StatList: public Token {
private:
    std::vector<std::shared_ptr<Statement>> stats;
public:
    StatList();
    StatList(std::shared_ptr<Statement> a);
    StatList(std::shared_ptr<StatList> a);
    StatList(std::shared_ptr<Statement> a, std::shared_ptr<Statement> b);
    StatList(std::shared_ptr<StatList> a, std::shared_ptr<Statement> b);
    StatList(std::shared_ptr<StatList> a, std::shared_ptr<StatList> b);
    void append(std::shared_ptr<Statement> a);
    void append(std::shared_ptr<StatList> a);
    std::vector<std::shared_ptr<Statement>> &getStatements();
    virtual void convert(convert::ScopeSP constext);
};

}
}

#endif /* _WAVEGUIDE_GRAMMAR_STATEMENTS_H_ */