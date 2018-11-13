#ifndef _WAVEGUIDE_GRAMMAR_OPERATORS_H_
#define _WAVEGUIDE_GRAMMAR_OPERATORS_H_

#include "Expressions.h"

namespace waveguide {
namespace grammar {

class OperatorExp: public Expression {
private:
    std::vector<std::shared_ptr<Expression>> args;
protected:
    virtual convert::ScopeSP getFunc() = 0;
public:
    void addArg(std::shared_ptr<Expression> arg);
    void addArgRec(std::shared_ptr<Expression> arg);
    virtual convert::ValueSP getValue(convert::ScopeSP context);
};

class AddExp: public OperatorExp {
protected:
    virtual convert::ScopeSP getFunc();
public:
    AddExp(std::shared_ptr<Expression> a, std::shared_ptr<Expression> b);
};

class MulExp: public OperatorExp {
protected:
    virtual convert::ScopeSP getFunc();
public:
    MulExp(std::shared_ptr<Expression> a, std::shared_ptr<Expression> b);
};

class ModExp: public OperatorExp {
protected:
    virtual convert::ScopeSP getFunc();
public:
    ModExp(std::shared_ptr<Expression> a, std::shared_ptr<Expression> b);
};

class IncExp: public OperatorExp {
protected:
    virtual convert::ScopeSP getFunc();
public:
    IncExp(std::shared_ptr<Expression> a);
};

class DecExp: public OperatorExp {
protected:
    virtual convert::ScopeSP getFunc();
public:
    DecExp(std::shared_ptr<Expression> a);
};

class RecipExp: public OperatorExp {
protected:
    virtual convert::ScopeSP getFunc();
public:
    RecipExp(std::shared_ptr<Expression> a);
};

#define OP_EXP_HELP(NAME) class NAME: public OperatorExp { \
protected: \
    virtual convert::ScopeSP getFunc(); \
public: \
    NAME(std::shared_ptr<Expression> a, std::shared_ptr<Expression> b); \
};

OP_EXP_HELP(EqExp);
OP_EXP_HELP(NeqExp);
OP_EXP_HELP(LteExp);
OP_EXP_HELP(GteExp);
OP_EXP_HELP(LtExp);
OP_EXP_HELP(GtExp);
OP_EXP_HELP(AndExp);
OP_EXP_HELP(OrExp);
OP_EXP_HELP(XorExp);
OP_EXP_HELP(BandExp);
OP_EXP_HELP(BorExp);
OP_EXP_HELP(BxorExp);

#undef OP_EXP_HELP

}
}

#endif /* _WAVEGUIDE_GRAMMAR_OPERATORS_H_ */