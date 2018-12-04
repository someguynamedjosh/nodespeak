#pragma once

#include <waveguide/parser/ast.hpp>

namespace waveguide {
namespace parser {

struct parse_result {
    ast::root_type ast;
    int error = 0;
};

parse_result parse(std::string input);

}
}