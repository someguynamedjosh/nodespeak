#pragma once

#include <boost/spirit/home/x3.hpp>
#include <boost/spirit/home/x3/support/utility/annotate_on_success.hpp>

#include <waveguide/parser/ast.hpp>
#include "ast_adapted.hpp"
#include "parser.hpp"
#include "parser_error.hpp"

namespace waveguide {
namespace parser {

namespace x3 = boost::spirit::x3;
namespace ascii = boost::spirit::x3::ascii;

using x3::rule;

////////////////////////////////////////////////////////////////////////////////
// Rule declarations
////////////////////////////////////////////////////////////////////////////////

#define RULE(RULE_NAME, ATTRIBUTE_TYPE) \
    struct RULE_NAME##_class; \
    rule<RULE_NAME##_class, ATTRIBUTE_TYPE> const \
        RULE_NAME = #RULE_NAME; \
    struct RULE_NAME##_class : x3::position_tagged { }

RULE(logic1_expr, ast::Expression);
RULE(logic2_expr, ast::Expression);
RULE(logic3_expr, ast::Expression);
RULE(blogic1_expr, ast::Expression);
RULE(blogic2_expr, ast::Expression);
RULE(blogic3_expr, ast::Expression);
RULE(equal_expr, ast::Expression);
RULE(compare_expr, ast::Expression);
RULE(add_expr, ast::Expression);
RULE(multiply_expr, ast::Expression);
RULE(signed_expr, ast::Expression);
RULE(basic_expr, ast::Expression);
RULE(array_expr, std::vector<ast::Expression>);
RULE(variable_expr, ast::VariableExpression);
RULE(function_expression_output, ast::FunctionExpressionOutput);
RULE(function_expr, ast::FunctionExpression);
RULE(noin_function_expr, ast::FunctionExpression);
RULE(justl_function_expr, ast::FunctionExpression);
RULE(default_function_expr, ast::FunctionExpression);
auto expr = logic1_expr; // Top-level expression.

RULE(data_type, ast::DataType);

RULE(statement, ast::Statement);
RULE(function_statement, ast::FunctionStatement);
RULE(assign_statement, ast::AssignStatement);
RULE(var_dec_statement, ast::VarDecStatement);
RULE(return_statement, ast::ReturnStatement);

RULE(function_input_dec, ast::FunctionParameterDec);
RULE(function_single_output_dec, ast::FunctionParameterDec);
RULE(function_dec, ast::FunctionDec);

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
using x3::eol;
using x3::string;
using x3::repeat;
using x3::lexeme;
using x3::lit;

// Used to 'cast' an attribute of a rule.
template <typename T> 
static auto as = [](auto p) { return x3::rule<struct tag, T> {"as"} = p; };

// Logic expressions
auto const logic1_expr_def = as<ast::OperatorListExpression>(
    logic2_expr >> *(
        (string("or") > logic2_expr)
    )
);

auto const logic2_expr_def = as<ast::OperatorListExpression>(
    logic3_expr >> *(
        (string("xor") > logic3_expr)
    )
);

auto const logic3_expr_def = as<ast::OperatorListExpression>(
    blogic1_expr >> *(
        (string("and") > blogic1_expr)
    )
);

// Bitwise logic expression
auto const blogic1_expr_def = as<ast::OperatorListExpression>(
    blogic2_expr >> *(
        (string("bor") > blogic2_expr)
    )
);

auto const blogic2_expr_def = as<ast::OperatorListExpression>(
    blogic3_expr >> *(
        (string("bxor") > blogic3_expr)
    )
);

auto const blogic3_expr_def = as<ast::OperatorListExpression>(
    equal_expr >> *(
        (string("band") > equal_expr)
    )
);

// Equality expression: ==, !=
auto const equal_expr_def = as<ast::OperatorListExpression>(
    compare_expr >> *(
        (string("==") > compare_expr)
        | (string("!=") > compare_expr)
    )
);

// Comparison expression: >=, <, etc.
auto const compare_expr_def = as<ast::OperatorListExpression>(
    add_expr >> *(
        (string(">=") > add_expr)
        | (string("<=") > add_expr)
        | (string(">") > add_expr)
        | (string("<") > add_expr)
    )
);

// Addition expressions: a + b - c + d etc.
auto const add_expr_def = as<ast::OperatorListExpression>(
    multiply_expr >> *(
        (string("+") > multiply_expr)
        | (string("-") > multiply_expr)
    )
);

// Multiplication expressions: a * b / c * d etc.
auto const multiply_expr_def = as<ast::OperatorListExpression>(
    signed_expr >> *(
        (string("*") > signed_expr)
        | (string("/") > signed_expr)
        | (string("%") > signed_expr)
    )
);

// Expressions with +/- signs.
auto const signed_expr_def =
    basic_expr
    | as<ast::SignedExpression>(char_('+') > basic_expr)
    | as<ast::SignedExpression>(char_('-') > basic_expr);

// Basic expressions: 1, 1.0, false, ({expression}), etc.
auto const basic_expr_def = 
    double_
    | int_
    | bool_
    | ('(' > expr > ')')
    | array_expr
    | function_expr
    | variable_expr;

// Array expression
auto const array_expr_def =
    '[' > expr % ',' > ']';

// Variable access
auto const variable_expr_def =
    identifier >> *('[' > expr > ']');

// Output of a function expression
auto const function_expression_output_def = 
    data_type >> identifier
    | variable_expr;

// Function calls.
auto const function_expr_def = 
    justl_function_expr | noin_function_expr | default_function_expr;

auto const justl_function_expr_def = (
    identifier
        >> repeat(0)[expr]
        >> repeat(0)[function_expression_output]
        >> +function_dec
);

auto const noin_function_expr_def = (
    identifier
        >> repeat(0)[expr]
        >> (lit(':') > '(' > -(function_expression_output % ',') > ')')
        >> *function_dec
);

auto const default_function_expr_def = (
    identifier
        >> ('(' > -(expr % ',') > ')')
        >> -(lit(':') > '(' > -(function_expression_output % ',') > ')')
        >> *function_dec
);



auto const data_type_def = 
    identifier >> *('[' > expr > ']');



auto const statement_def =
    return_statement | var_dec_statement | function_statement 
    | assign_statement;

auto const function_statement_def =
    function_expr >> ';';

auto const assign_statement_def =
    variable_expr >> '=' > expr > ';';

auto const var_dec_statement_def =
    data_type >> as<ast::VarDec>(
        as<ast::InitVarDec>(identifier >> '=' > expr)
        | as<ast::PlainVarDec>(identifier)
    ) % ',' >> ';';

auto const return_statement_def =
    "return" > expr > ';';



auto const function_input_dec_def =
    data_type > identifier;

auto const function_single_output_dec_def =
    (data_type >> attr("return"));

auto const function_dec_def = 
    identifier 
        >> -('(' > -(function_input_dec % ',') > ')') 
        >> -(lit(':') > (
            ('(' > -(function_input_dec % ',') > ')')
            | repeat(1)[function_single_output_dec]
        ))
        >> -('[' > -(function_dec % ',') > ']')
        >> ('{' > *statement > '}');



auto const identifier_def =
    lexeme[(alpha | '_') >> *(alnum | '_')];

auto const skipper =
    lit(' ') | '\t' | '\n' | lexeme['#' > *(char_ - eol) >> eol];

auto const root_rule_def =
   *statement;



BOOST_SPIRIT_DEFINE(logic1_expr, logic2_expr, logic3_expr, blogic1_expr, 
    blogic2_expr, blogic3_expr, equal_expr, compare_expr, add_expr, 
    multiply_expr, signed_expr, basic_expr, array_expr, variable_expr, 
    function_expr, function_expression_output, noin_function_expr, 
    justl_function_expr, default_function_expr)
BOOST_SPIRIT_DEFINE(data_type)
BOOST_SPIRIT_DEFINE(statement, function_statement, assign_statement, 
    var_dec_statement, return_statement)
BOOST_SPIRIT_DEFINE(function_input_dec, function_single_output_dec, 
    function_dec)
BOOST_SPIRIT_DEFINE(identifier, root_rule)

}
}