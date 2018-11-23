#pragma once

#include <boost/spirit/home/x3.hpp>
#include <boost/spirit/home/x3/support/utility/annotate_on_success.hpp>

#include "ast.hpp"
#include "ast_adapted.hpp"
#include "parser.hpp"

namespace waveguide {
namespace parser {

namespace x3 = boost::spirit::x3;
namespace ascii = boost::spirit::x3::ascii;

using x3::rule;

////////////////////////////////////////////////////////////////////////////////
// Rule declarations
////////////////////////////////////////////////////////////////////////////////

#define RULE(RULE_NAME, ATTRIBUTE_TYPE) \
    rule<struct RULE_NAME##_class, ATTRIBUTE_TYPE> const \
        RULE_NAME = #RULE_NAME

RULE(logic_expr, ast::Expression);
RULE(blogic_expr, ast::Expression);
RULE(equal_expr, ast::Expression);
RULE(compare_expr, ast::Expression);
RULE(add_expr, ast::Expression);
RULE(multiply_expr, ast::Expression);
RULE(signed_expr, ast::Expression);
RULE(basic_expr, ast::Expression);
RULE(variable_expr, ast::VariableExpression);
RULE(function_expr, ast::FunctionExpression);
auto expr = logic_expr; // Top-level expression.

RULE(data_type, ast::DataType);
RULE(array_data_type, ast::ArrayDataType);
RULE(plain_data_type, ast::PlainDataType);

RULE(statement, ast::Statement);
RULE(function_statement, ast::FunctionStatement);
RULE(assign_statement, ast::AssignStatement);
RULE(var_dec_statement, ast::VarDecStatement);

RULE(identifier, std::string);
root_rule_type const root_rule = "root_rule";

#undef RULE

////////////////////////////////////////////////////////////////////////////////
// Rule definitions
////////////////////////////////////////////////////////////////////////////////

using x3::double_;
using x3::int_;
using x3::bool_;
using x3::char_;
using x3::string;
using x3::alpha;
using x3::alnum;

using x3::attr;
using x3::string;
using x3::repeat;
using x3::lexeme;

// Used to 'cast' an attribute of a rule.
template <typename T> 
static auto as = [](auto p) { return x3::rule<struct tag, T> {"as"} = p; };

// Logic expressions
auto const logic_expr_def = as<ast::OperatorListExpression>(
    blogic_expr >> *(
        string("and") >> blogic_expr
        | string("or") >> blogic_expr
        | string("xor") >> blogic_expr
    )
);

// Bitwise logic expression
auto const blogic_expr_def = as<ast::OperatorListExpression>(
    equal_expr >> *(
        string("band") >> equal_expr
        | string("bor") >> equal_expr
        | string("bxor") >> equal_expr
    )
);

// Equality expression: ==, !=
auto const equal_expr_def = as<ast::OperatorListExpression>(
    compare_expr >> *(
        string("==") >> compare_expr
        | string("!=") >> compare_expr
    )
);

// Comparison expression: >=, <, etc.
auto const compare_expr_def = as<ast::OperatorListExpression>(
    add_expr >> *(
        string(">") >> add_expr
        | string("<") >> add_expr
        | string(">=") >> add_expr
        | string("<=") >> add_expr
    )
);

// Addition expressions: a + b - c + d etc.
auto const add_expr_def = as<ast::OperatorListExpression>(
    multiply_expr >> *(
        string("+") >> multiply_expr
        | string("/") >> multiply_expr
    )
);

// Multiplication expressions: a * b / c * d etc.
auto const multiply_expr_def = as<ast::OperatorListExpression>(
    signed_expr >> *(
        string("*") >> signed_expr
        | string("/") >> signed_expr 
        | string("%") >> signed_expr
    )
);

// Expressions with +/- signs.
auto const signed_expr_def =
    basic_expr
    | as<ast::SignedExpression>(char_('+') >> basic_expr)
    | as<ast::SignedExpression>(char_('-') >> basic_expr);

// Basic expressions: 1, 1.0, false, ({expression}), etc.
auto const basic_expr_def = 
    int_
    | double_
    | bool_
    | '(' >> expr >> ')'
    | function_expr
    | variable_expr;

// Variable access
auto const variable_expr_def =
    as<ast::VariableExpression>(identifier);

// Function calls.
auto const function_expr_def =
    as<ast::FunctionExpression>(identifier >> '(' >> (expr % ',') >> ')');



auto const data_type_def = 
    plain_data_type | array_data_type;

auto const array_data_type_def =
    data_type >> '[' >> expr >> ']';

auto const plain_data_type_def =
    identifier;



auto const statement_def =
    function_statement | assign_statement | var_dec_statement;

auto const function_statement_def =
    function_expr >> ';';

auto const assign_statement_def =
    variable_expr >> '=' >> expr >> ';';

auto const var_dec_statement_def =
    data_type >> as<ast::VarDec>(
        as<ast::InitVarDec>(identifier >> '=' >> expr)
        | as<ast::PlainVarDec>(identifier)
    ) % ',' >> ';';



auto const identifier_def =
    lexeme[(alpha | '_') >> *(alnum | '_')];

auto const root_rule_def =
   statement;



BOOST_SPIRIT_DEFINE(logic_expr, blogic_expr, equal_expr, compare_expr, add_expr, 
    multiply_expr, signed_expr, basic_expr, variable_expr, function_expr)
BOOST_SPIRIT_DEFINE(data_type, array_data_type, plain_data_type)
BOOST_SPIRIT_DEFINE(statement, function_statement, assign_statement, var_dec_statement)
BOOST_SPIRIT_DEFINE(identifier, root_rule)

}
}