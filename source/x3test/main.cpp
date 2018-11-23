#include <iostream>
#include <string>

#include "ast.hpp"
#include "ast_adapted.hpp"
#include "parser.hpp"

int main() {
    std::string code = "1";
    std::cout << "Start" << std::endl;
    auto result = waveguide::parser::parse(code);
    std::cout << "End" << std::endl;
    return 0;
}