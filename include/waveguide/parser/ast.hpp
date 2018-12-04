#pragma once

#include <boost/spirit/home/x3/support/ast/position_tagged.hpp>
#include <boost/spirit/home/x3/support/ast/variant.hpp>
#include <string>
#include <vector>

namespace waveguide {
namespace ast {

namespace x3 = boost::spirit::x3;

struct FunctionStatement;
struct AssignStatement;
struct VarDecStatement;
struct ReturnStatement;

using StatementVariant = x3::variant<
    x3::forward_ast<FunctionStatement>,
    x3::forward_ast<AssignStatement>,
    x3::forward_ast<VarDecStatement>,
    x3::forward_ast<ReturnStatement>>;
struct Statement: StatementVariant, x3::position_tagged {
    using base_type::base_type;
    using base_type::operator=;
    void operator=(Statement const&stat) { base_type::operator=(stat); }
    Statement(Statement &stat) : StatementVariant(stat) { }
    Statement(Statement const&stat) : StatementVariant(stat) { }
};

struct FunctionExpression;
struct OperatorListExpression;
struct SignedExpression;
struct VariableExpression;

struct Expression: x3::variant<
    int, double, bool, 
    x3::forward_ast<std::vector<Expression>>,
    x3::forward_ast<FunctionExpression>, 
    x3::forward_ast<VariableExpression>,
    x3::forward_ast<OperatorListExpression>, 
    x3::forward_ast<SignedExpression>>, x3::position_tagged {
    using base_type::base_type;
    using base_type::operator=;
};



struct DataType: x3::position_tagged {
    std::string name;
    std::vector<Expression> array_sizes;
};



struct FunctionParameterDec: x3::position_tagged {
    DataType type;
    std::string name;
};

struct FunctionDec: x3::position_tagged {
    std::string name;
    std::vector<FunctionParameterDec> inputs, outputs;
    std::vector<x3::forward_ast<FunctionDec>> lambdas;
    std::vector<Statement> body;
};



struct OperatorExpression: x3::position_tagged {
    std::string op_char;
    Expression value;
};

struct OperatorListExpression: x3::position_tagged {
    Expression start_value;
    std::vector<OperatorExpression> operations;
};

struct SignedExpression: x3::position_tagged {
    char sign;
    Expression value;
};

struct VariableExpression: x3::position_tagged {
    std::string name;
    std::vector<Expression> array_accesses;
};

struct SingleVarDec: x3::position_tagged {
    DataType type;
    std::string name;
};

using FSOVariant = x3::variant<SingleVarDec, VariableExpression>;
struct FunctionExpressionOutput: FSOVariant {
    using base_type::base_type;
    using base_type::operator=;
    void operator=(FunctionExpressionOutput const&expr) 
        { base_type::operator=(expr); }
    FunctionExpressionOutput(FunctionExpressionOutput &expr): 
        FSOVariant(expr) { }
    FunctionExpressionOutput(FunctionExpressionOutput const&expr): 
        FSOVariant(expr) { }
};

struct FunctionExpression: x3::position_tagged {
    std::string function_name;
    std::vector<Expression> inputs;
    std::vector<FunctionExpressionOutput> outputs;
    std::vector<FunctionDec> lambdas;
};



struct FunctionStatement: x3::position_tagged {
    FunctionExpression func_call;
};

struct AssignStatement: x3::position_tagged {
    VariableExpression assign_to;
    Expression value;
};

struct PlainVarDec: x3::position_tagged {
    std::string name;
};

struct InitVarDec: x3::position_tagged {
    std::string name;
    Expression value;
};

struct VarDec: x3::variant<PlainVarDec, InitVarDec>, x3::position_tagged {
    using base_type::base_type;
    using base_type::operator=;
    void operator=(VarDec const&dec) { base_type::operator=(dec); }
    VarDec(VarDec &dec) : x3::variant<PlainVarDec, InitVarDec>(dec) { }
    VarDec(VarDec const&dec) : x3::variant<PlainVarDec, InitVarDec>(dec) { }
};

struct VarDecStatement: x3::position_tagged {
    DataType type;
    std::vector<VarDec> var_decs;
};

struct ReturnStatement: x3::position_tagged {
    Expression value;
};

using root_type = std::vector<ast::Statement>;

}
}