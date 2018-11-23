#include <iostream>
#include <string>

#include "ast.hpp"
#include "ast_adapted.hpp"
#include "parser.hpp"

int main() {
    std::string code = "(12321.0e12) + (123 * 123 / 123 + 123)";
    std::cout << "Start" << std::endl;
    auto result = waveguide::parser::parse(code);
    std::cout << "End" << std::endl;
    return 0;
}