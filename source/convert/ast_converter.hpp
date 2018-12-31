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

inline SP<intr::builtins> blt() {
    return intr::builtins::get_instance();
}

inline SP<intr::value> int_literal(const int value) {
    return SP<intr::value>{new intr::value(blt()->INT, new int{value})};
}

inline SP<intr::value> float_literal(const float value) {
    return SP<intr::value>{new intr::value(blt()->FLOAT, new float{value})};
}

inline SP<intr::value> bool_literal(const bool value) {
    return SP<intr::value>{new intr::value(blt()->BOOL, new bool{value})};
}

struct access_result {
    SP<intr::value> root_val, offset;
    SP<const intr::data_type> final_type;
};

struct ast_converter_data {
    bool fpd_is_input;
    SP<intr::scope> current_scope;
    SP<intr::value> current_value;
    SP<intr::data_type> current_type;
    SP<intr::vague_data_type> current_vtype;
    SP<intr::vague_expression> current_vexpr;
};

struct ast_converter: util::static_visitor<ast_converter, ast_converter_data> {
    virtual void on_start() const override;
    SP<intr::scope> get_result() const;

    // Utility methods.
    access_result find_access_result(ast::variable_expression const&expr) const;
    void copy_value_to_expr(SP<intr::value> from, 
        ast::variable_expression const& to) const;
    void copy_value_from_expr(ast::variable_expression const& from,
        SP<intr::value> to) const;
    SP<intr::value> lookup_var(std::string name) const;
    SP<intr::scope> lookup_func(std::string name) const;
    SP<intr::data_type> lookup_type(std::string name) const;
    void add_command(SP<intr::command> command) const;
    void declare_temp_var(SP<intr::value> var) const;
    
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