#pragma once

#include <waveguide/parser/ast.hpp>
#include <exception>
#include <memory>
#include <ostream>
#include <string>

namespace waveguide {
namespace intermediate {

class scope;
std::ostream &operator<<(std::ostream &stream, scope const&to_print);

}
}

namespace waveguide {
namespace convert {

struct conversion_result {
    std::shared_ptr<intermediate::scope> converted_scope;
    bool success;
    std::string error_message;
};

conversion_result convert_ast(ast::root_type const&root);

}
}