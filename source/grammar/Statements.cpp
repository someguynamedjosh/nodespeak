#include "Statements.h"

namespace waveguide {
namespace grammar {

////////////////////////////////////////////////////////////////////////////////
// FuncCallStat, VarDec, AssignStat, FuncDec, Branch, ForLoop, WhileLoop
////////////////////////////////////////////////////////////////////////////////

FuncCallStat::FuncCallStat(std::shared_ptr<FuncCall> call)
    : call{call} { }

VarDec::VarDec(std::shared_ptr<DataType> type, std::string name)
    : type{type}, name{name} { }

std::shared_ptr<DataType> VarDec::getType() {
    return type;
}

std::string VarDec::getName() {
    return name;
}

AssignStat::AssignStat(std::shared_ptr<AccessExp> to,
    std::shared_ptr<Expression> value)
    : to{to}, value{value} { }

FuncDec::FuncDec(std::string name, std::shared_ptr<StatList> ins,
    std::shared_ptr<StatList> outs, std::shared_ptr<StatList> body)
    : name{name}, ins{ins}, outs{outs}, body{body} { }

std::string FuncDec::getName() {
    return name;
}

std::shared_ptr<StatList> FuncDec::getBody() {
    return body;
}

Branch::Branch(std::shared_ptr<Expression> con,
    std::shared_ptr<StatList> ifTrue)
    : con{con}, ifTrue{ifTrue} { }

ForLoop::ForLoop(std::shared_ptr<VarDec> counter, 
    std::shared_ptr<ExpList> values, std::shared_ptr<StatList> body)
    : counter{counter}, values{values}, body{body} { }

WhileLoop::WhileLoop(std::shared_ptr<Expression> condition,
    std::shared_ptr<StatList> body)
    : condition{condition}, body{body} { }

////////////////////////////////////////////////////////////////////////////////
// StatList
////////////////////////////////////////////////////////////////////////////////
StatList::StatList() { }

StatList::StatList(std::shared_ptr<Statement> a)
    : stats{a} { }

StatList::StatList(std::shared_ptr<StatList> a) {
    append(a);
}

StatList::StatList(std::shared_ptr<Statement> a, std::shared_ptr<Statement> b)
    : stats{a, b} { }

StatList::StatList(std::shared_ptr<StatList> a, std::shared_ptr<Statement> b) {
    append(a);
    append(b);
}

StatList::StatList(std::shared_ptr<StatList> a, std::shared_ptr<StatList> b) {
    append(a);
    append(b);
}

void StatList::append(std::shared_ptr<Statement> a) {
    stats.push_back(a);
}

void StatList::append(std::shared_ptr<StatList> a) {
    for (auto stat : a->stats) {
        stats.push_back(stat);
    }
}

std::vector<std::shared_ptr<Statement>> &StatList::getStatements() {
    return stats;
}

}
}
