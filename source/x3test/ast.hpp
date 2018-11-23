#pragma once

#include <boost/spirit/home/x3/support/ast/position_tagged.hpp>
#include <boost/spirit/home/x3/support/ast/variant.hpp>
#include <string>
#include <vector>

namespace waveguide {
namespace ast {

namespace x3 = boost::spirit::x3;

struct FunctionExpression;
struct OperatorListExpression;
struct SignedExpression;
struct VariableExpression;

struct Expression: x3::variant<
    int, double, bool, 
    x3::forward_ast<FunctionExpression>, 
    x3::forward_ast<VariableExpression>,
    x3::forward_ast<OperatorListExpression>, 
    x3::forward_ast<SignedExpression>> {
    using base_type::base_type;
    using base_type::operator=;
};

struct FunctionExpression {
    std::string functionName;
    std::vector<Expression> inputs;
};

struct OperatorExpression {
    std::string op_char;
    Expression value;
};

struct OperatorListExpression {
    Expression start_value;
    std::vector<OperatorExpression> operations;
};

struct SignedExpression {
    char sign;
    Expression value;
};

struct VariableExpression {
    std::string name;
};

}
}