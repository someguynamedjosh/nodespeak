#pragma once

#include "util/aliases.hpp"

#include <waveguide/intermediate/scope.hpp>
#include <memory>

namespace waveguide {
namespace squash {

intr::resolved_scope_ptr cast_pass(intr::scope_ptr scope);

}
}