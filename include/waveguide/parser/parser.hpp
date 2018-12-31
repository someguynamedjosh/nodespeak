#pragma once

#include <waveguide/parser/ast.hpp>

namespace waveguide {
namespace parser {

struct parse_result {
    bool success;
    ast::root_type ast;
};

parse_result parse(std::string input);

}
}