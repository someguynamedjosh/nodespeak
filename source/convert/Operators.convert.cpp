#include "grammar/Operators.h"

namespace waveguide {
namespace convert {

ValueSP grammar::OperatorExp::getValue(ScopeSP context) {
    ScopeSP call = getFunc();
    CommandSP command{new Command{call}};
    for (auto expr : args) {
        command->addInput(expr->getValue(context));
    }

    ValueSP tvar{new Value{call->getOuts()[0]->getType()}};
    context->declareTempVar(tvar);
    command->addOutput(tvar);
    context->addCommand(command);
    return tvar;
}

#define OP_FUNC_HELP(OP_NAME, FUNC_NAME) \
ScopeSP grammar::OP_NAME::getFunc() { \
    return blt()->FUNC_NAME; \
}

OP_FUNC_HELP(AddExp, ADD);
OP_FUNC_HELP(MulExp, MUL);
OP_FUNC_HELP(ModExp, MOD);
// TODO: These two are not correct.
OP_FUNC_HELP(IncExp, ADD);
OP_FUNC_HELP(DecExp, ADD);
OP_FUNC_HELP(RecipExp, RECIP);

OP_FUNC_HELP(EqExp, EQ);
OP_FUNC_HELP(NeqExp, NEQ);
OP_FUNC_HELP(LteExp, LTE);
OP_FUNC_HELP(GteExp, GTE);
OP_FUNC_HELP(LtExp, LT);
OP_FUNC_HELP(GtExp, GT);
OP_FUNC_HELP(AndExp, AND);
OP_FUNC_HELP(OrExp, OR);
OP_FUNC_HELP(XorExp, XOR);
OP_FUNC_HELP(BandExp, BAND);
OP_FUNC_HELP(BorExp, BOR);
OP_FUNC_HELP(BxorExp, BXOR);

}
}