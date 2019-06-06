#pragma once

#include <waveguide/_shared/value.hpp>

namespace waveguide {
namespace resolved {

class data_type;

using value = _shared::value<data_type>;
using value_accessor = _shared::value_accessor<value>;
using const_data_block_ptr = _shared::const_data_block_ptr;
using data_block_ptr = _shared::data_block_ptr;
using shared_data_block_ptr = _shared::shared_data_block_ptr;

}
}