#pragma once

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

class builtins;
class scope;
class data_type;

SP<data_type> biggest_type(SP<data_type> a, SP<data_type> b);
SP<builtins> blt();

}
}