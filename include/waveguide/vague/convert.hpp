#pragma once

#include <memory>
#include <string>
#include <waveguide/ast/types.hpp>

namespace waveguide {
namespace vague {

class scope;

typedef std::shared_ptr<scope> scope_ptr;

struct conversion_result {
    scope_ptr converted_scope;
    bool success;
    std::string error_message;
};

conversion_result convert_ast(ast::root_type const&root);

}
}