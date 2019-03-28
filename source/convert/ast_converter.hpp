#pragma once

#include <waveguide/convert/convert.hpp>
#include <waveguide/intermediate/builtins.hpp>
#include <waveguide/intermediate/metastructure.hpp>
#include <waveguide/parser/ast.hpp>
#include <memory>

#include "util/aliases.hpp"
#include "util/static_visitor.hpp"

namespace waveguide {
namespace convert {

class ast_conversion_exception: public std::exception {
private:
    std::string message;
public:
    ast_conversion_exception(std::string message);
    const char *what() const throw();
};

}
}

namespace waveguide {
namespace ast {

inline intr::builtins_ptr blt() {
    return intr::builtins::get_instance();
}

inline intr::value_accessor_ptr int_literal(const int value) {
    auto literal = std::make_shared<intr::value>(
        blt()->INT, std::make_shared<int>(value)
    );
    literal->set_debug_label("Literal " + std::to_string(value));
    return std::make_shared<intr::value_accessor>(literal);
}

inline intr::value_accessor_ptr float_literal(const float value) {
    auto literal = std::make_shared<intr::value>(
        blt()->FLOAT, std::make_shared<float>(value)
    );
    literal->set_debug_label("Literal " + std::to_string(value));
    return std::make_shared<intr::value_accessor>(literal);
}

inline intr::value_accessor_ptr bool_literal(const bool value) {
    auto literal = std::make_shared<intr::value>(
        blt()->BOOL, std::make_shared<bool>(value)
    );
    literal->set_debug_label("Literal " + std::to_string(value));
    return std::make_shared<intr::value_accessor>(literal);
}

inline intr::value_accessor_ptr access(intr::value_ptr value) {
    return std::make_shared<intr::value_accessor>(value);
}

struct access_result {
    intr::value_ptr root_val, offset;
    intr::const_data_type_ptr final_type;
};

struct ast_converter_data {
    bool fpd_is_input, is_lambda;
    intr::scope_ptr current_scope;
    intr::value_accessor_ptr current_value;
    intr::data_type_ptr current_type;
    intr::vague_data_type_ptr current_vtype;
    intr::vague_expression_ptr current_vexpr;
};

struct ast_converter: util::static_visitor<ast_converter, ast_converter_data> {
    virtual void on_start() const override;
    intr::scope_ptr get_result() const;

    // Utility methods.
    access_result find_access_result(ast::variable_expression const&expr) const;
    void copy_value_to_expr(intr::value_ptr from, 
        ast::variable_expression const& to) const;
    void copy_value_from_expr(ast::variable_expression const& from,
        intr::value_ptr to) const;
    intr::value_ptr lookup_var(std::string name) const;
    intr::scope_ptr lookup_func(std::string name) const;
    intr::data_type_ptr lookup_type(std::string name) const;
    void add_command(intr::command_ptr command) const;
    void declare_temp_var(intr::value_ptr) const;
    
    // Parses statements into the current scope.
    void operator()(std::vector<statement> const&stats) const;
    void operator()(function_statement const&stat) const;
    void operator()(assign_statement const&stat) const;
    void operator()(var_dec_statement const&stat) const;
    void operator()(plain_var_dec const&dec) const;
    void operator()(init_var_dec const&dec) const;
    void operator()(return_statement const&stat) const;

    void operator()(int const&expr) const;
    void operator()(float const&expr) const;
    void operator()(bool const&expr) const;
    void operator()(signed_expression const&expr) const;
    void operator()(variable_expression const&expr) const;
    void operator()(std::vector<expression> const&expr) const;
    void operator()(single_var_dec const&dec) const;
    void operator()(function_expression const&expr) const;
    void operator()(operator_list_expression const&expr) const;

    void operator()(function_parameter_dec const&dec) const;
    void operator()(function_dec const&dec) const;
    void operator()(data_type const&type) const;
    void operator()(vague_data_type const&type) const;
    void operator()(vague_number_expression const&type) const;
    void operator()(vague_variable_expression const&expr) const;
    void operator()(vague_signed_expression const&expr) const;
    void operator()(vague_operator_list_expression const&expr) const;
};

}
}