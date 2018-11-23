#include <iostream>
#include <string>

#include "ast.hpp"
#include "ast_adapted.hpp"
#include "ast_printer.hpp"
#include "parser.hpp"

int main() {
    std::string code = "Int a, b, c;";
    auto result = waveguide::parser::parse(code);
    waveguide::ast::print_ast(result);
    return 0;
}