#pragma once

#include <iostream>

#include "ast.hpp"

namespace waveguide {
namespace ast {

constexpr int INDENT_WIDTH = 4;

struct AstPrinter: boost::static_visitor<> {
    int indent;

    AstPrinter(int indent): indent(indent) { }

    void print_indent() const {
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

    void operator()(PlainDataType const&type) const {
        std::cout << "type:" << type.name << std::endl;
    }

    void operator()(ArrayDataType const&type) const {
        recurse(type.base);
        std::cout << '[';
        recurse(type.size);
        std::cout << ']';
    }

    void operator()(FunctionStatement const&stat) const {
        print_indent();
        (*this)(stat.func_call);
        std::cout << ';' << std::endl;
    }

    void operator()(AssignStatement const&stat) const {
        print_indent();
        (*this)(stat.assign_to);
        std::cout << " = ";
        recurse(stat.value);
        std::cout << ';' << std::endl;
    }

    void operator()(PlainVarDec const&dec) const {
        std::cout << dec.name;
    }

    void operator()(InitVarDec const&dec) const {
        std::cout << dec.name << " = ";
        recurse(dec.value);
    }

    void operator()(VarDecStatement const&stat) const {
        print_indent();
        std::cout << "declare, ";
        recurse(stat.type);
        bool first = true;
        for (auto const&dec : stat.var_decs) {
            if (!first) std::cout << ", ";
            first = false;
            recurse(dec);
        }
        std::cout << ';' << std::endl;
    }

    template<typename Visitable>
    void recurse(Visitable &to_print) const {
        boost::apply_visitor(AstPrinter{indent + INDENT_WIDTH}, to_print);
    }
};

template<typename Visitable>
inline void print_ast(Visitable const& expr) {
    boost::apply_visitor(AstPrinter{0}, expr);
}

}
}