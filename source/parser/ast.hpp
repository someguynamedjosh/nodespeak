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



struct PlainDataType;
struct ArrayDataType;

struct DataType: x3::variant<
    x3::forward_ast<PlainDataType>,
    x3::forward_ast<ArrayDataType>> {
    using base_type::base_type;
    using base_type::operator=;
};

struct PlainDataType {
    std::string name;
};

struct ArrayDataType {
    DataType base;
    Expression size;
};



struct FunctionStatement;
struct AssignStatement;
struct VarDecStatement;

struct Statement: x3::variant<
    x3::forward_ast<FunctionStatement>,
    x3::forward_ast<AssignStatement>,
    x3::forward_ast<VarDecStatement>> {
    using base_type::base_type;
    using base_type::operator=;
};

struct FunctionStatement {
    FunctionExpression func_call;
};

struct AssignStatement {
    VariableExpression assign_to;
    Expression value;
};

struct PlainVarDec {
    std::string name;
};

struct InitVarDec {
    std::string name;
    Expression value;
};

struct VarDec: x3::variant<PlainVarDec, InitVarDec> {
    using base_type::base_type;
    using base_type::operator=;
    void operator=(VarDec const&dec) { base_type::operator=(dec); }
    VarDec(VarDec &dec) : x3::variant<PlainVarDec, InitVarDec>(dec) { }
    VarDec(VarDec const&dec) : x3::variant<PlainVarDec, InitVarDec>(dec) { }
};

struct VarDecStatement {
    DataType type;
    std::vector<VarDec> var_decs;
};

}
}