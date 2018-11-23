#include <iostream>
#include <string>
#include <vector>

#include "waveguide/parser.hpp"
#include "waveguide/ast_util.hpp"

int main() {
    std::vector<std::string> tests{
        "Int a;", "Int a, b;", "Int a = 1, b;", "Int a, b = 1;", 
        "Int a, b = 1, c;", "test_func(1);", "f(1.0);", "f(0.1);", "f(.1);",
        "f(true);", "f(false);", "f(test_var);", "f(1 + 1);", "f(1 - 1);", 
        "f(1 * 1);", "f(1 / 1);", "f(1 % 1);", "f(1 * 1 / 1 % 1);", 
        "f(1 + 1 - 1);", "f(1 + 1 * 1 + 1);", "f(1 == 1 and 1 != 1);", 
        "f(1 and 1 or 1 xor 1);", "f(1 band 1 bor 1 bxor 1);", 
        "f(1 > 1 and 1 < 1 and 1 >= 1 and 1 <= 1);", "f(g(h(test_var)));",
        "Int a = 1;", "Int[12] b;", "Int[12][12] c;", "Int[12+4][4] d;",
        "a = 1;", "b = 123 + 456;", "c = f(g(d + 12 / 34));"
    };

    uint successes = 0;
    for (auto const&test : tests) {
        auto result = waveguide::parser::parse(test);
        if (result.error) {
            std::cout << "=====ERROR PARSING TEST!=====" << std::endl;
            std::cout << "Input:" << std::endl;
            std::cout << test << std::endl;
            std::cout << "=============================" << std::endl;
        } else {
            successes++;
        }
    }

    std::cout << std::endl << successes << '/' << tests.size() << ' ';
    std::cout << "Completed sucessfully." << std::endl;
    return successes != tests.size() ? 1 : 0;
}