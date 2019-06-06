#pragma once

#include <waveguide/ast/types.hpp>

namespace waveguide {
namespace parser {

struct parse_result {
    bool success;
    ast::root_type ast;
};

parse_result parse(std::string input);

}
}