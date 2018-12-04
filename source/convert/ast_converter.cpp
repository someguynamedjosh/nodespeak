#include "ast_converter.hpp"

#include <waveguide/convert/convert.hpp>

namespace waveguide {
namespace convert {

void convert_ast(std::vector<ast::Statement> const&root) {
    ast::AstConverter{}(root);
}

}
}