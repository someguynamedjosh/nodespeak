#include "ast_printer.hpp"

#include <waveguide/ast/ast_printer.hpp>

namespace waveguide {
namespace ast {

void print_ast(root_type const&root) {
    ast_printer{0}(root);
}

}
}
