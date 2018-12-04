#include "ast_converter.hpp"

#include <waveguide/convert/convert.hpp>

namespace waveguide {
namespace convert {

void convert_ast(ast::root_type const&root) {
    ast::AstConverter{}(root);
}

}
}