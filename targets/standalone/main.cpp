#include <fstream>
#include <iostream>
#include <string>

#include "waveguide/parser.hpp"
#include "waveguide/ast_util.hpp"

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
    return 0;
}