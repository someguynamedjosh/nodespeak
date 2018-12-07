#pragma once

#include <waveguide/parser/ast.hpp>
#include <memory>

namespace waveguide {
namespace intermediate {

class scope;

}
}

namespace waveguide {
namespace convert {

std::shared_ptr<intermediate::scope> convert_ast(ast::root_type const&root);

}
}