#include "Expressions.h"

namespace waveguide {
namespace grammar {

////////////////////////////////////////////////////////////////////////////////
// IdentifierExp, IntExp, FloatExp, BoolExp
////////////////////////////////////////////////////////////////////////////////
IdentifierExp::IdentifierExp(std::string name)
    : name{name} { }

IntExp::IntExp(int value)
    : value{value} { }

FloatExp::FloatExp(float value)
    : value{value} { }

BoolExp::BoolExp(bool value)
    : value{value} { }

////////////////////////////////////////////////////////////////////////////////
// AccessExp
////////////////////////////////////////////////////////////////////////////////
AccessExp::AccessExp(std::shared_ptr<IdentifierExp> rootVar)
    : rootVar{rootVar} { }

void AccessExp::addIndexAccessor(std::shared_ptr<Expression> index) {
    accessors.push_back(index);
}

void AccessExp::addMemberAccessor(std::shared_ptr<std::string> member) {
    accessors.push_back(member);
}

////////////////////////////////////////////////////////////////////////////////
// ExpList
////////////////////////////////////////////////////////////////////////////////
ExpList::ExpList(std::shared_ptr<Expression> a)
    : exps{a} { }

ExpList::ExpList(std::shared_ptr<Expression> a, std::shared_ptr<Expression> b)
    : exps{a, b} { }

ExpList::ExpList(std::shared_ptr<Expression> a, std::shared_ptr<ExpList> b)
    : exps{a} {
    for (auto expression : b->getExps()) {
        exps.push_back(expression);
    }
}

////////////////////////////////////////////////////////////////////////////////
// ArrayLiteral
////////////////////////////////////////////////////////////////////////////////
ArrayLiteral::ArrayLiteral(std::shared_ptr<ExpList> elements)
    : elements{elements} { }

////////////////////////////////////////////////////////////////////////////////
// Range
////////////////////////////////////////////////////////////////////////////////
Range::Range(std::shared_ptr<Expression> start, std::shared_ptr<Expression> end)
    : start{start}, end{end} { }

Range::Range(std::shared_ptr<Expression> start, std::shared_ptr<Expression> end,
    std::shared_ptr<Expression> step)
    : start{start}, end{end}, step{step} { }

////////////////////////////////////////////////////////////////////////////////
// Output, RetOut, NoneOut, VarAccessOut
////////////////////////////////////////////////////////////////////////////////
std::shared_ptr<Expression> Output::getExp() {
    return nullptr;
}

int RetOut::getType() {
    return TYPE_CONST;
}

int NoneOut::getType() {
    return TYPE_CONST;
}

VarAccessOut::VarAccessOut(std::shared_ptr<Expression> exp)
    : exp(exp) { }

int VarAccessOut::getType() {
    return TYPE_CONST;
}

////////////////////////////////////////////////////////////////////////////////
// OutList
////////////////////////////////////////////////////////////////////////////////
OutList::OutList() { }

OutList::OutList(std::shared_ptr<Output> a)
    : outputs{a} { }

OutList::OutList(std::shared_ptr<Output> a, std::shared_ptr<Output> b)
    : outputs{a, b} { }

void OutList::append(std::shared_ptr<Output> a) {
    outputs.push_back(a);
}

void OutList::append(std::shared_ptr<OutList> a) {
    for (auto output : a->getOutputs()) {
        outputs.push_back(output);
    }
}

std::vector<std::shared_ptr<Output>> &OutList::getOutputs() {
    return outputs;
}

////////////////////////////////////////////////////////////////////////////////
// FuncCall
////////////////////////////////////////////////////////////////////////////////
FuncCall::FuncCall(std::string name, std::shared_ptr<ExpList> ins, 
    std::shared_ptr<OutList> outs)
    : name{name}, ins{ins}, outs{outs} { }

}
}