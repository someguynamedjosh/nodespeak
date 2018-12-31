#include "passes.hpp"

#include <waveguide/squash/squash.hpp>

namespace waveguide {
namespace squash {

void squash(SP<intr::scope> scope) {
    cast_pass(scope);
}

}
}