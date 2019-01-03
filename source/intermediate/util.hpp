#pragma once

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

class builtins;
class scope;
class data_type;

typedef std::shared_ptr<builtins> builtins_ptr;
typedef std::shared_ptr<data_type> data_type_ptr;

data_type_ptr biggest_type(data_type_ptr a, data_type_ptr b);
builtins_ptr blt();

}
}