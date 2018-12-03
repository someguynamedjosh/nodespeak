#pragma once

#include "intermediate/builtins.hpp"
#include "intermediate/scope.hpp"
#include "intermediate/value.hpp"
#include "parser/ast.hpp"

namespace waveguide {
namespace ast {

namespace intr = waveguide::intermediate;
template<typename T>
using SP = std::shared_ptr<T>;

inline SP<intr::Builtins> blt() {
    return intr::Builtins::get_instance();
}

inline SP<intr::Value> int_literal(const int value) {
    return SP<intr::Value>{new intr::Value(blt()->INT, new int{value})};
}

struct AccessResult {
    SP<intr::Value> root_val, offset;
    SP<intr::DataType> final_type;
};

struct AstConverter: boost::static_visitor<> {
    struct ConverterData {
        SP<intr::Scope> current_scope;
        SP<intr::Value> current_value;
        SP<intr::DataType> current_type;
    };
    SP<ConverterData> data;
    mutable std::vector<SP<ConverterData>> stack;

    AstConverter();
    AstConverter(SP<ConverterData> data): data{data} { }

    // Utility methods.
    void push_stack() const;
    void pop_stack() const;
    AccessResult find_access_result(ast::VariableExpression const&expr) const;
    void copy_value_to_expr(SP<intr::Value> from, 
        ast::VariableExpression const& to) const;
    void copy_value_from_expr(ast::VariableExpression const& from,
        SP<intr::Value> to) const;
    SP<intr::Value> lookup_var(std::string name) const;
    SP<intr::Scope> lookup_func(std::string name) const;
    SP<intr::DataType> lookup_type(std::string name) const;
    void add_command(SP<intr::Command> command) const;
    void declare_temp_var(SP<intr::Value> var) const;
    
    // Parses statements into the current scope.
    void operator()(std::vector<Statement> const&stats) const;
    void operator()(FunctionStatement const&stat) const;
    void operator()(AssignStatement const&stat) const;
    void operator()(VarDecStatement const&stat) const;
    void operator()(PlainVarDec const&dec) const;
    void operator()(InitVarDec const&dec) const;
    void operator()(ReturnStatement const&stat) const;

    void operator()(FunctionParameterDec const&dec) const;
    void operator()(FunctionDec const&dec) const;
    void operator()(int const&expr) const;
    void operator()(double const&expr) const;
    void operator()(bool const&expr) const;
    void operator()(std::vector<Expression> const&expr) const;
    void operator()(SingleVarDec const&dec) const;
    void operator()(FunctionExpression const&expr) const;
    void operator()(OperatorListExpression const&expr) const;
    void operator()(SignedExpression const&expr) const;
    void operator()(VariableExpression const&expr) const;
    void operator()(DataType const&type) const;

    template<typename Visitable>
    void recurse(Visitable &to_convert) const {
        boost::apply_visitor(AstConverter{data}, to_convert);
    }
};

template<typename Visitable>
inline void convert_ast(Visitable const&expr) {
    boost::apply_visitor(AstConverter{}, expr);
}

template<>
inline void convert_ast(std::vector<ast::Statement> const&root) {
    AstConverter{}(root);
}

}
}