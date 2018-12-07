#include "ast_converter.hpp"

#include <waveguide/convert/convert.hpp>
#include <waveguide/intermediate/metastructure.hpp>

namespace waveguide {
namespace convert {

SP<intr::scope> convert_ast(ast::root_type const&root) {
    ast::AstConverter converter{};
    converter.start(root);
    return converter.get_result();
}

}
}