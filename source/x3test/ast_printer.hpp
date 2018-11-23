#pragma once

#include <iostream>

#include "ast.hpp"

namespace waveguide {
namespace ast {

constexpr int INDENT_WIDTH = 4;

struct AstPrinter: boost::static_visitor<> {
    int indent;

    AstPrinter(int indent): indent(indent) { }

    void write_indent() const {
        for (int i = 0; i < indent; i++) {
            std::cout << " ";
        }
    }

    void operator()(int const&expr) const {
        std::cout << expr;
    }

    void operator()(double const&expr) const {
        std::cout << expr;
    }

    void operator()(bool const&expr) const {
        std::cout << (expr ? "true" : "false");
    }

    void operator()(FunctionExpression const&expr) const {
        std::cout << expr.functionName << '(';
        bool first = true;
        for (auto const&input : expr.inputs) {
            if (!first) std::cout << ", ";
            first = false;
            recurse(input);
        }
        std::cout << ')';
    }

    void operator()(OperatorListExpression const&expr) const {
        if (expr.operations.size() == 0) {
            recurse(expr.start_value);
            return;
        }
        std::cout << '(';
        recurse(expr.start_value);
        for (auto const&operation : expr.operations) {
            std::cout << ' ' << operation.op_char << ' ';
            recurse(operation.value);
        }
        std::cout << ')';
    }

    void operator()(SignedExpression const&expr) const {
        std::cout << '(' << expr.sign << ')';
        recurse(expr.value);
    }

    void operator()(VariableExpression const&expr) const {
        std::cout << expr.name;
    }

    template<typename Visitable>
    void recurse(Visitable &to_print) const {
        boost::apply_visitor(AstPrinter{indent + INDENT_WIDTH}, to_print);
    }
};

inline void print_ast(Expression const& expr) {
    boost::apply_visitor(AstPrinter{0}, expr);
}

}
}