#pragma once

#include <boost/fusion/include/adapt_struct.hpp>

#include "ast.hpp"

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::FunctionExpression,
    functionName, inputs
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::OperatorExpression,
    op_char, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::SignedExpression,
    sign, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::OperatorListExpression,
    start_value, operations
)
