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

    void operator()(FunctionParameterDec const&dec) const {
        (*this)(dec.type);
        std::cout << " " << dec.name;
    }

    void operator()(FunctionDec const&dec) const {
        std::cout << dec.name << "(";
        bool first = true;
        for (auto const&input : dec.inputs) {
            if (!first) std::cout << ", ";
            first = false;
            (*this)(input);
        }
        std::cout << "):(";
        first = true;
        for (auto const&output : dec.outputs) {
            if (!first) std::cout << ", ";
            first = false;
            (*this)(output);
        }
        std::cout << ") [";
        first = true;
        for (auto const&lambda : dec.lambdas) {
            if (!first) std::cout << ", ";
            first = false;
            (*this)(lambda);
        }
        std::cout << "] { ";
        if (dec.body.size() > 0) std::cout << std::endl;
        for (auto const&stat : dec.body) {
            recurse(stat);
        }
        if (dec.body.size() > 0) print_indent();
        std::cout << "}";
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

    void operator()(SingleVarDec const&dec) const {
        std::cout << "declare, ";
        (*this)(dec.type);
        std::cout << " " << dec.name << " ";
    }

    void operator()(FunctionExpression const&expr) const {
        std::cout << expr.functionName << '(';
        bool first = true;
        for (auto const&input : expr.inputs) {
            if (!first) std::cout << ", ";
            first = false;
            recurse(input);
        }
        std::cout << "):(";
        first = true;
        for (auto const&output : expr.outputs) {
            if (!first) std::cout << ", ";
            first = false;
            recurse(output);
        }
        std::cout << ") [";
        first = true;
        for (auto const&lambda : expr.lambdas) {
            if (!first) std::cout << ", ";
            first = false;
            (*this)(lambda);
        }
        std::cout << "]";
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

    void operator()(DataType const&type) const {
        std::cout << type.name;
        for (auto &size : type.array_sizes) {
            std::cout << '[';
            recurse(size);
            std::cout << ']';
        }
    }

    void operator()(std::vector<Statement> const&stats) const {
        for (auto const&stat : stats) {
            recurse(stat);
        }
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
        (*this)(stat.type);
        std::cout << " ";
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
inline void print_ast(Visitable const&expr) {
    boost::apply_visitor(AstPrinter{0}, expr);
}

template<>
inline void print_ast(std::vector<ast::Statement> const&root) {
    AstPrinter{0}(root);
}

}
}