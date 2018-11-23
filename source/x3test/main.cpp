#include <iostream>
#include <string>

#include "ast.hpp"
#include "ast_adapted.hpp"
#include "parser.hpp"

int main() {
    std::string code = "sin(one + two)";
    auto result = waveguide::parser::parse(code);
    return 0;
}