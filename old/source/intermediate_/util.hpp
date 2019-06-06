#pragma once

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

class builtins;
class scope;
class data_type;

typedef std::shared_ptr<builtins> builtins_ptr;
typedef std::shared_ptr<const data_type> const_data_type_ptr;

const_data_type_ptr biggest_type(const_data_type_ptr a, const_data_type_ptr b);
builtins_ptr blt();

}
}