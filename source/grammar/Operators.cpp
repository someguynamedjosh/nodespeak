#include "Operators.h"

namespace waveguide {
namespace grammar {

////////////////////////////////////////////////////////////////////////////////
// OperatorExp
////////////////////////////////////////////////////////////////////////////////
void OperatorExp::addArg(std::shared_ptr<Expression> arg) {
    args.push_back(arg);
}

void OperatorExp::addArgRec(std::shared_ptr<Expression> arg) {
    if (typeid(arg.get()) == typeid(this)) {
        for (auto subArg : std::dynamic_pointer_cast<OperatorExp>(arg)->args) {
            args.push_back(subArg);
        }
    } else {
        args.push_back(arg);
    }
}

////////////////////////////////////////////////////////////////////////////////
// AddExp, MulExp, ModExp, IncExp, DecExp, RecipExp
////////////////////////////////////////////////////////////////////////////////
AddExp::AddExp(std::shared_ptr<Expression> a, std::shared_ptr<Expression> b) {
    addArgRec(a);
    addArgRec(b);
}

MulExp::MulExp(std::shared_ptr<Expression> a, std::shared_ptr<Expression> b) {
    addArgRec(a);
    addArgRec(b);
}

ModExp::ModExp(std::shared_ptr<Expression> a, std::shared_ptr<Expression> b) {
    addArg(a);
    addArg(b);
}

IncExp::IncExp(std::shared_ptr<Expression> a) {
    addArg(a);
}

DecExp::DecExp(std::shared_ptr<Expression> a) {
    addArg(a);
}

RecipExp::RecipExp(std::shared_ptr<Expression> a) {
    addArg(a);
}

////////////////////////////////////////////////////////////////////////////////
// Macro-generated exps
////////////////////////////////////////////////////////////////////////////////
#define OP_EXP_HELP(NAME) \
NAME::NAME(std::shared_ptr<Expression> a, std::shared_ptr<Expression> b) { \
    addArg(a); \
    addArg(b); \
}

OP_EXP_HELP(EqExp);
OP_EXP_HELP(NeqExp);
OP_EXP_HELP(Lte);
OP_EXP_HELP(Gte);
OP_EXP_HELP(Lt);
OP_EXP_HELP(Gt);
OP_EXP_HELP(And);
OP_EXP_HELP(Or);
OP_EXP_HELP(Xor);
OP_EXP_HELP(Band);
OP_EXP_HELP(Bor);
OP_EXP_HELP(Bxor);

}
}