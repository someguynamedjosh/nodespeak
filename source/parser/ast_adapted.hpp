#pragma once

#include <boost/fusion/include/adapt_struct.hpp>

#include <waveguide/parser/ast.hpp>

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::data_type,
    array_sizes, name
)



BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::function_parameter_dec,
    type, name
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::function_dec,
    name, inputs, outputs, lambdas, body
)



BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::operator_expression,
    op_char, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::signed_expression,
    sign, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::variable_expression,
    name, array_accesses
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::operator_list_expression,
    start_value, operations
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::single_var_dec,
    type, name
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::function_expression,
    function_name, inputs, outputs, lambdas
)



BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::vague_data_type,
    array_sizes, name, is_unknown
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::vague_number_expression,
    value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::vague_variable_expression,
    name, is_unknown
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::vague_signed_expression,
    sign, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::vague_operator_expression,
    op_char, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::vague_operator_list_expression,
    start_value, operations
)



BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::function_statement,
    func_call
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::assign_statement,
    assign_to, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::plain_var_dec,
    name
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::init_var_dec,
    name, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::var_dec_statement,
    type, var_decs
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::return_statement,
    value
)