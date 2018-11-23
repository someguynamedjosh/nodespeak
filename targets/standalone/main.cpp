#include <iostream>
#include <string>

#include "waveguide/parser.hpp"
#include "waveguide/ast_util.hpp"

int main() {
    std::string code = "Int a, b, c;";
    auto result = waveguide::parser::parse(code);
    std::cout << 
        (result.error ? "Compile failed!" : "Compile suceeded!") << std::endl;
    waveguide::ast::print_ast(result.ast);
    return 0;
}