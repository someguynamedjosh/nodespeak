#ifndef _WAVEGUIDE_GRAMMAR_EXPRESSIONS_H_
#define _WAVEGUIDE_GRAMMAR_EXPRESSIONS_H_

#include <memory>
#include <variant>

#include "Token.h"

namespace waveguide {
namespace grammar {

class Expression: public Token {
public:
    virtual convert::ValueSP getValue(convert::ScopeSP context) = 0;
};

class IdentifierExp: public Expression {
protected:
    std::string name;
public:
    IdentifierExp(std::string name);
    virtual convert::ValueSP getValue(convert::ScopeSP context);
};

class IntExp: public Expression {
protected:
    int value;
public:
    IntExp(int value);
    virtual convert::ValueSP getValue(convert::ScopeSP context);
};

class FloatExp: public Expression {
protected:
    float value;
public:
    FloatExp(float value);
    virtual convert::ValueSP getValue(convert::ScopeSP context);
};

class BoolExp: public Expression {
protected:
    bool value;
public:
    BoolExp(bool value);
    virtual convert::ValueSP getValue(convert::ScopeSP context);
};

class AccessExp: public Expression {
private:
    std::shared_ptr<IdentifierExp> rootVar;
    typedef std::variant<std::shared_ptr<Expression>,
                         std::shared_ptr<std::string>> Accessor;
    std::vector<Accessor> accessors;
    struct AccessResult { 
        convert::ValueSP rootVal, offset; 
        convert::DTypeSP finalType;
    };
public:
    AccessExp(std::shared_ptr<IdentifierExp> rootVar);
    void addIndexAccessor(std::shared_ptr<Expression> index);
    void addMemberAccessor(std::shared_ptr<std::string> member);
    virtual convert::ValueSP getValue(convert::ScopeSP context);
    void setFromValue(convert::ScopeSP context, convert::ValueSP copyFrom);
};

class ExpList: public Token {
private:
    std::vector<std::shared_ptr<Expression>> exps;
public:
    ExpList(std::shared_ptr<Expression> a);
    ExpList(std::shared_ptr<Expression> a, std::shared_ptr<Expression> b);
    ExpList(std::shared_ptr<Expression> a, std::shared_ptr<ExpList> b);
    void append(std::shared_ptr<Expression> a);
    void append(std::shared_ptr<ExpList> a);
    std::vector<std::shared_ptr<Expression>> &getExps();
};

class ArrayLiteral: public Expression {
private:
    std::shared_ptr<ExpList> elements;
public:
    ArrayLiteral(std::shared_ptr<ExpList> elements);
    virtual convert::ValueSP getValue(convert::ScopeSP context);
};

class Range: public Expression {
private:
    std::shared_ptr<Expression> start, end, step{nullptr};
public:
    Range(std::shared_ptr<Expression> start, std::shared_ptr<Expression> end);
    Range(std::shared_ptr<Expression> start, std::shared_ptr<Expression> end,
        std::shared_ptr<Expression> step);
    virtual convert::ValueSP getValue(convert::ScopeSP context);
};

class Output: public Token {
public: 
    virtual int getType() = 0;
    virtual std::shared_ptr<Expression> getExp();
};

class RetOut: public Output {
public:
    static const int TYPE_CONST = 0;
    virtual int getType();
};

class NoneOut: public Output {
public:
    static const int TYPE_CONST = 1;
    virtual int getType();
};

class VarAccessOut: public Output {
private:
    std::shared_ptr<Expression> exp;
public:
    static const int TYPE_CONST = 2;
    VarAccessOut(std::shared_ptr<Expression> exp);
    virtual int getType();
    virtual std::shared_ptr<Expression> getExp();
};

class OutList: public Token {
private:
    std::vector<std::shared_ptr<Output>> outputs;
public:
    OutList();
    OutList(std::shared_ptr<Output> a);
    OutList(std::shared_ptr<Output> a, std::shared_ptr<Output> b);
    void append(std::shared_ptr<Output> a);
    void append(std::shared_ptr<OutList> a);
    std::vector<std::shared_ptr<Output>> &getOutputs();
};

class FuncCall: public Expression {
protected:
    std::string name;
    std::shared_ptr<ExpList> ins;
    std::shared_ptr<OutList> outs;
public:
    FuncCall(std::string name, std::shared_ptr<ExpList> ins, 
        std::shared_ptr<OutList> outs);
    virtual std::shared_ptr<Expression> getExp();
};

}
}

#endif /* _WAVEGUIDE_GRAMMAR_EXPRESSIONS_H_ */