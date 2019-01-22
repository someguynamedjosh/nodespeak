#include "passes.hpp"

#include <waveguide/squash/squash.hpp>

namespace waveguide {
namespace squash {

squash_result squash(intr::scope_ptr scope) {
    squash_result result{};
    result.squashed = cast_pass(scope);
    return result;
}

}
}