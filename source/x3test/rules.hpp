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

RULE(add_expr, ast::Expression);
RULE(multiply_expr, ast::Expression);
RULE(signed_expr, ast::Expression);
RULE(basic_expr, ast::Expression);

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
using x3::attr;
using x3::string;
using x3::repeat;

// Used to 'cast' an attribute of a rule.
template <typename T> 
static auto as = [](auto p) { return x3::rule<struct tag, T> {"as"} = p; };


// Addition expressions: a + b - c + d etc.
auto const add_expr_def = as<ast::OperatorListExpression>(
    multiply_expr >> *(
        string("+") >> basic_expr
        | string("/") >> basic_expr
    )
);

// Multiplication expressions: a * b / c * d etc.
auto const multiply_expr_def = as<ast::OperatorListExpression>(
    signed_expr >> *(
        string("*") >> basic_expr
        | string("/") >> basic_expr
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
    | '(' >> add_expr >> ')';

auto const root_rule_def =
    add_expr;


BOOST_SPIRIT_DEFINE(add_expr, multiply_expr, signed_expr, basic_expr, 
    root_rule)

}
}