#include <fstream>
#include <iostream>
#include <sstream>
#include <string>

#include <waveguide/compile.hpp>
#include <waveguide/util.hpp>

int main(int argc, char **argv) {
    if (argc != 2) {
        std::cout << "Usage:" << std::endl;
        std::cout << "waveguide_standalone [source]" << std::endl;
        std::cout << "source is either a file name to retrieve code from, or a";
        std::cout << "dash to read from cin." << std::endl;
        return 1;
    }
    std::string code = "";
    if (argv[1][0] == '-' && argv[1][1] == '\x00') {
        std::cout << "Reading code from stdin." << std::endl;
        std::string line;
        while (std::getline(std::cin, line)) {
            code += line + "\n";
        }
    } else {
        std::cout << "Reading code from file " << argv[1] << "." << std::endl;
        std::ifstream file{argv[1]};
        std::stringstream temp_buf;
        temp_buf << file.rdbuf();
        code = temp_buf.str();
    }
    std::cout << "Compiling code:" << std::endl;
    std::cout << code << std::endl;
    auto result = waveguide::parser::parse(code);
    std::cout << 
        (result.error ? "Compile failed!" : "Compile suceeded!") << std::endl;
    waveguide::ast::print_ast(result.ast);
    auto conversion_result = waveguide::convert::convert_ast(result.ast);
    if (!conversion_result->success) {
        std::cerr << "Error converting AST!" << std::endl;
        std::cerr << conversion_result->error_message << std::endl;
        return 1;
    }
    waveguide::squash::squash(conversion_result->converted_scope);
    std::cout << *conversion_result->converted_scope.get() << std::endl;
    return 0;
}