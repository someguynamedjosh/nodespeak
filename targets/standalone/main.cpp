#include <iostream>
#include <string>

#include "waveguide/parser.hpp"
#include "waveguide/ast_util.hpp"

int main() {
    std::string code = "Int a, b, c;";
    auto result = waveguide::parser::parse(code);
    waveguide::ast::print_ast(result);
    return 0;
}