#include "passes.hpp"

#include <waveguide/squash/squash.hpp>

namespace waveguide {
namespace squash {

squash_result squash(intr::scope_ptr scope) {
    cast_pass(scope);
    return squash_result{};
}

}
}