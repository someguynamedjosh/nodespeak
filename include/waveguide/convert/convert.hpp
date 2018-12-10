#pragma once

#include <waveguide/parser/ast.hpp>
#include <memory>
#include <ostream>

namespace waveguide {
namespace intermediate {

class scope;
std::ostream &operator<<(std::ostream &stream, scope const&to_print);

}
}

namespace waveguide {
namespace convert {

std::shared_ptr<intermediate::scope> convert_ast(ast::root_type const&root);

}
}