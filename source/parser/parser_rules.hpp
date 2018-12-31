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

#define OPERATOR_EXPRESSION_RULE(EXPRESSION_NAME) \
    RULE(EXPRESSION_NAME##_expr, ast::operator_list_expression); \
    RULE(EXPRESSION_NAME##_op, ast::operator_expression)

OPERATOR_EXPRESSION_RULE(logic1);
OPERATOR_EXPRESSION_RULE(logic2);
OPERATOR_EXPRESSION_RULE(logic3);
OPERATOR_EXPRESSION_RULE(blogic1);
OPERATOR_EXPRESSION_RULE(blogic2);
OPERATOR_EXPRESSION_RULE(blogic3);
OPERATOR_EXPRESSION_RULE(equal);
OPERATOR_EXPRESSION_RULE(compare);
OPERATOR_EXPRESSION_RULE(add);
OPERATOR_EXPRESSION_RULE(multiply);

RULE(signed_expr, ast::expression);
RULE(basic_expr, ast::expression);
RULE(array_expr, std::vector<ast::expression>);
RULE(variable_expr, ast::variable_expression);
RULE(function_expression_output, ast::function_expression_output);
RULE(function_expr, ast::function_expression);
RULE(noin_function_expr, ast::function_expression);
RULE(justl_function_expr, ast::function_expression);
RULE(default_function_expr, ast::function_expression);
auto expr = logic1_expr; // Top-level expression.

RULE(data_type, ast::data_type);

RULE(vague_add_expr, ast::vague_expression);
RULE(vague_multiply_expr, ast::vague_expression);
RULE(vague_signed_expr, ast::vague_expression);
RULE(vague_variable_expr, ast::vague_variable_expression);
RULE(vague_basic_expr, ast::vague_expression);
RULE(vague_number_expr, ast::vague_number_expression);
auto vague_expr = vague_add_expr; // Top-level vague expression.
RULE(vague_data_type, ast::vague_data_type);

RULE(statement, ast::statement);
RULE(function_statement, ast::function_statement);
RULE(assign_statement, ast::assign_statement);
RULE(var_dec_statement, ast::var_dec_statement);
RULE(return_statement, ast::return_statement);

RULE(function_input_dec, ast::function_parameter_dec);
RULE(function_single_output_dec, ast::function_parameter_dec);
RULE(function_dec, ast::function_dec);

RULE(identifier, std::string);
root_rule_type const root_rule = "root_rule";

////////////////////////////////////////////////////////////////////////////////
// Rule definitions
////////////////////////////////////////////////////////////////////////////////

using x3::float_;
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
auto const logic1_expr_def = logic2_expr >> *logic1_op;
auto const logic1_op_def = string("or") > logic2_expr;

auto const logic2_expr_def = logic3_expr >> *logic2_op;
auto const logic2_op_def = string("xor") > logic3_expr;

auto const logic3_expr_def = blogic1_expr >> *logic3_op;
auto const logic3_op_def = string("and") > blogic1_expr;

// Bitwise logic expression
auto const blogic1_expr_def = blogic2_expr >> *blogic1_op;
auto const blogic1_op_def = string("bor") > blogic2_expr;

auto const blogic2_expr_def = blogic3_expr >> *blogic2_op;
auto const blogic2_op_def = string("bxor") > blogic3_expr;

auto const blogic3_expr_def = equal_expr >> *blogic3_op;
auto const blogic3_op_def = string("band") > equal_expr;

// Equality expression: ==, !=
auto const equal_expr_def = compare_expr >> *equal_op;
auto const equal_op_def =
    (string("==") > compare_expr)
    | (string("!=") > compare_expr);

// Comparison expression: >=, <, etc.
auto const compare_expr_def = add_expr >> *compare_op;
auto const compare_op_def =
    (string(">=") > add_expr)
    | (string("<=") > add_expr)
    | (string(">") > add_expr)
    | (string("<") > add_expr);

// Addition expressions: a + b - c + d etc.
auto const add_expr_def = multiply_expr >> *add_op;
auto const add_op_def =
    (string("+") > multiply_expr)
    | (string("-") > multiply_expr);

// Multiplication expressions: a * b / c * d etc.
auto const multiply_expr_def = signed_expr >> *multiply_op;
auto const multiply_op_def =
    (string("*") > signed_expr)
    | (string("/") > signed_expr)
    | (string("%") > signed_expr);

// expressions with +/- signs.
auto const signed_expr_def =
    basic_expr
    | as<ast::signed_expression>(char_('+') > basic_expr)
    | as<ast::signed_expression>(char_('-') > basic_expr);

// Basic expressions: 1, 1.0, false, ({expression}), etc.
auto const basic_expr_def = 
    x3::real_parser<float, x3::strict_real_policies<float>>{}
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
    *('[' > expr > ']') >> identifier;



auto const vague_add_expr_def = as<ast::vague_operator_list_expression>(
    vague_multiply_expr >> *(
        (string("+") > vague_multiply_expr)
        | (string("-") > vague_multiply_expr)
    )
);

auto const vague_multiply_expr_def = as<ast::vague_operator_list_expression>(
    vague_signed_expr >> *(
        (string("*") > vague_signed_expr)
        | (string("/") > vague_signed_expr)
        | (string("%") > vague_signed_expr)
    )
);

auto const vague_signed_expr_def = 
    vague_basic_expr
    | as<ast::vague_signed_expression>(char_('-') > vague_basic_expr);

auto const vague_basic_expr_def = 
    vague_number_expr
    | ('(' > vague_expr > ')')
    | vague_variable_expr;

auto const vague_number_expr_def =
    int_;

auto const vague_variable_expr_def =
    identifier >> -char_('?');

auto const vague_data_type_def =
    *('[' > vague_expr > ']') >> identifier >> -char_('?');



auto const statement_def =
    return_statement | var_dec_statement | function_statement 
    | assign_statement;

auto const function_statement_def =
    function_expr >> ';';

auto const assign_statement_def =
    variable_expr >> '=' > expr > ';';

auto const var_dec_statement_def =
    data_type >> as<ast::var_dec>(
        as<ast::init_var_dec>(identifier >> '=' > expr)
        | as<ast::plain_var_dec>(identifier)
    ) % ',' >> ';';

auto const return_statement_def =
    "return" > expr > ';';



auto const function_input_dec_def =
    vague_data_type > identifier;

auto const function_single_output_dec_def =
    (vague_data_type >> attr("return"));

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



BOOST_SPIRIT_DEFINE(logic1_expr, logic1_op, logic2_expr, logic2_op, logic3_expr,
    logic3_op, blogic1_expr, blogic1_op, blogic2_expr, blogic2_op, blogic3_expr,
    blogic3_op, equal_expr, equal_op, compare_expr, compare_op, add_expr,
    add_op, multiply_expr, multiply_op)
BOOST_SPIRIT_DEFINE(signed_expr, basic_expr, array_expr, variable_expr, 
    function_expr, function_expression_output, noin_function_expr, 
    justl_function_expr, default_function_expr)
BOOST_SPIRIT_DEFINE(data_type)
BOOST_SPIRIT_DEFINE(vague_add_expr, vague_multiply_expr, vague_signed_expr,
    vague_basic_expr, vague_number_expr, vague_variable_expr, vague_data_type)
BOOST_SPIRIT_DEFINE(statement, function_statement, assign_statement, 
    var_dec_statement, return_statement)
BOOST_SPIRIT_DEFINE(function_input_dec, function_single_output_dec, 
    function_dec)
BOOST_SPIRIT_DEFINE(identifier, root_rule)

}
}