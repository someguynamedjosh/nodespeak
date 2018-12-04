#pragma once

#include <waveguide/parser/ast.hpp>
#include <vector>

namespace waveguide {
namespace convert {

void convert_ast(std::vector<ast::Statement> const&root);

}
}