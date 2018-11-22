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

RULE(basic_expr, ast::Expression);
RULE(signed_expr, ast::Expression);

root_type const root = "root";

#undef RULE

////////////////////////////////////////////////////////////////////////////////
// Rule definitions
////////////////////////////////////////////////////////////////////////////////

using x3::double_;
using x3::int_;
using x3::bool_;
using x3::attr;
using x3::lit;

// Used to 'cast' an attribute of a rule.
template <typename T> 
static auto as = [](auto p) { return x3::rule<struct tag, T> {"as"} = p; };
#define as_fe as<ast::FunctionExpression>
#define as_ev as<std::vector<ast::Expression>>

auto const basic_expr_def = 
    double_
    | int_
    | bool_;

auto const signed_expr_def =
    basic_expr
    | '+' >> basic_expr
    | '-' >> as_fe(
        attr("!MUL") >> as_ev(
            attr(-1) >> basic_expr
        )
    );

auto const root_def =
    signed_expr;

BOOST_SPIRIT_DEFINE(basic_expr, signed_expr, root);

}

parser::root_type root() {
    return parser::root;
}

}