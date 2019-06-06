#pragma once

#include <waveguide/intermediate/scope.hpp>
#include <memory>

namespace waveguide {
namespace squash {

struct squash_result {
    bool success;
    std::string error_message;
    std::shared_ptr<intermediate::resolved_scope> squashed;
};

squash_result squash(std::shared_ptr<intermediate::scope> scope);

}
}