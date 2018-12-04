#include "ast_printer.hpp"

#include <waveguide/parser/ast_printer.hpp>

namespace waveguide {
namespace ast {

void print_ast(root_type const&root) {
    AstPrinter{0}(root);
}

}
}
