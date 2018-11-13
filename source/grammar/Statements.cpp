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

AssignStat::AssignStat(std::shared_ptr<AccessExp> to,
    std::shared_ptr<Expression> value)
    : to{to}, value{value} { }

FuncDec::FuncDec(std::string name, std::shared_ptr<StatList> ins,
    std::shared_ptr<StatList> outs, std::shared_ptr<StatList> body)
    : name{name}, ins{ins}, outs{outs}, body{body} { }

Branch::Branch(std::shared_ptr<Expression> con,
    std::shared_ptr<StatList> ifTrue)
    : con{con}, ifTrue{ifTrue} { }

ForLoop::ForLoop(std::shared_ptr<VarDec> counter, 
    std::shared_ptr<ExpList> values, std::shared_ptr<StatList> body)
    : counter{counter}, values{values}, body{body} { }

WhileLoop::WhileLoop(std::shared_ptr<Expression> condition,
    std::shared_ptr<StatList> body)
    : condition{condition}, body{body} { }

}
}
