#pragma once

#include <boost/spirit/home/x3/support/ast/position_tagged.hpp>
#include <boost/spirit/home/x3/support/ast/variant.hpp>
#include <string>
#include <vector>

namespace waveguide {
namespace ast {

namespace x3 = boost::spirit::x3;

struct function_statement;
struct assign_statement;
struct var_dec_statement;
struct return_statement;

using statement_variant = x3::variant<
    x3::forward_ast<function_statement>,
    x3::forward_ast<assign_statement>,
    x3::forward_ast<var_dec_statement>,
    x3::forward_ast<return_statement>>;
struct statement: statement_variant, x3::position_tagged {
    using base_type::base_type;
    using base_type::operator=;
    void operator=(statement const&stat) { base_type::operator=(stat); }
    statement(statement &stat) : statement_variant(stat) { }
    statement(statement const&stat) : statement_variant(stat) { }
};

struct function_expression;
struct operator_list_expression;
struct signed_expression;
struct variable_expression;

struct expression: x3::variant<
    int, float, bool, 
    x3::forward_ast<std::vector<expression>>,
    x3::forward_ast<function_expression>, 
    x3::forward_ast<variable_expression>,
    x3::forward_ast<operator_list_expression>, 
    x3::forward_ast<signed_expression>>, x3::position_tagged {
    using base_type::base_type;
    using base_type::operator=;
};



struct data_type: x3::position_tagged {
    std::string name;
    std::vector<expression> array_sizes;
};



struct function_parameter_dec: x3::position_tagged {
    data_type type;
    std::string name;
};

struct function_dec: x3::position_tagged {
    std::string name;
    std::vector<function_parameter_dec> inputs, outputs;
    std::vector<x3::forward_ast<function_dec>> lambdas;
    std::vector<statement> body;
};



struct operator_expression: x3::position_tagged {
    std::string op_char;
    expression value;
};

struct operator_list_expression: x3::position_tagged {
    expression start_value;
    std::vector<operator_expression> operations;
};

struct signed_expression: x3::position_tagged {
    char sign;
    expression value;
};

struct variable_expression: x3::position_tagged {
    std::string name;
    std::vector<expression> array_accesses;
};

struct single_var_dec: x3::position_tagged {
    data_type type;
    std::string name;
};

using fso_variant = x3::variant<single_var_dec, variable_expression>;
struct function_expression_output: fso_variant {
    using base_type::base_type;
    using base_type::operator=;
    void operator=(function_expression_output const&expr) 
        { base_type::operator=(expr); }
    function_expression_output(function_expression_output &expr): 
        fso_variant(expr) { }
    function_expression_output(function_expression_output const&expr): 
        fso_variant(expr) { }
};

struct function_expression: x3::position_tagged {
    std::string function_name;
    std::vector<expression> inputs;
    std::vector<function_expression_output> outputs;
    std::vector<function_dec> lambdas;
};



struct function_statement: x3::position_tagged {
    function_expression func_call;
};

struct assign_statement: x3::position_tagged {
    variable_expression assign_to;
    expression value;
};

struct plain_var_dec: x3::position_tagged {
    std::string name;
};

struct init_var_dec: x3::position_tagged {
    std::string name;
    expression value;
};

struct var_dec: x3::variant<plain_var_dec, init_var_dec>, x3::position_tagged {
    using base_type::base_type;
    using base_type::operator=;
    void operator=(var_dec const&dec) { base_type::operator=(dec); }
    var_dec(var_dec &dec) : x3::variant<plain_var_dec, init_var_dec>(dec) { }
    var_dec(var_dec const&dec) : x3::variant<plain_var_dec, init_var_dec>(dec) { }
};

struct var_dec_statement: x3::position_tagged {
    data_type type;
    std::vector<var_dec> var_decs;
};

struct return_statement: x3::position_tagged {
    expression value;
};

using root_type = std::vector<ast::statement>;

}
}