#include <iostream>
#include <string>

#include "waveguide/parser.hpp"
#include "waveguide/ast_util.hpp"

int main(int argc, char **argv) {
    std::string code = "";
    // First argument is program name / path. 
    for (int i = 1; i < argc; i++) {
        code += std::string{argv[i]} + " ";
    }
    std::cout << "Compiling code:" << std::endl;
    std::cout << code << std::endl;
    auto result = waveguide::parser::parse(code);
    std::cout << 
        (result.error ? "Compile failed!" : "Compile suceeded!") << std::endl;
    waveguide::ast::print_ast(result.ast);
    return 0;
}