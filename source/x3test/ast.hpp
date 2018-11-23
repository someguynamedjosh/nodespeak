#pragma once

#include <boost/spirit/home/x3/support/ast/position_tagged.hpp>
#include <boost/spirit/home/x3/support/ast/variant.hpp>
#include <string>
#include <vector>

namespace waveguide {
namespace ast {

namespace x3 = boost::spirit::x3;

struct Expression;

struct FunctionExpression {
    std::string functionName;
    std::vector<x3::forward_ast<Expression>> inputs;
};

struct OperatorExpression {
    std::string op_char;
    x3::forward_ast<Expression> value;
};

struct OperatorListExpression {
    x3::forward_ast<Expression> start_value;
    std::vector<OperatorExpression> operations;
};

struct SignedExpression {
    char sign;
    x3::forward_ast<Expression> value;
};

struct VariableExpression {
    std::string name;
};

struct Expression: x3::variant<int, double, bool, FunctionExpression, 
    OperatorListExpression, SignedExpression, VariableExpression> {
    using base_type::base_type;
    using base_type::operator=;
};

}
}