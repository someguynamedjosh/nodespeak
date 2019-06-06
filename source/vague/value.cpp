#include <waveguide/vague/value.hpp>

#include <waveguide/vague/type_template.hpp>
#include <cassert>

#include "util.hpp"

namespace waveguide {
namespace vague {

value_accessor::const_metatype_ptr value_accessor::get_type() const {
    auto data_type = root_value->get_type();
    for (auto subpart : subparts) {
        if (subpart->get_type() == blt()->INT) {
            data_type = std::dynamic_pointer_cast<
                const template_array_data_type
            >(data_type)->get_base_type();
        } else {
            // TODO: Add support for object keys.
            return nullptr;
        }
    }
    return data_type;
}

data_block_ptr value_accessor::get_data() {
    assert(root_value);
    assert(is_value_known());
    auto data_type = root_value->get_type();
    data_block_ptr ptr = root_value->get_data();

    // We cannot do array processing because we do not know the size of the base
    // type of the array yet. That's why it's vague.
    assert(subparts.size() == 0);

    return ptr;
}

const_data_block_ptr value_accessor::get_data() const {
    assert(root_value);
    assert(is_value_known());
    auto data_type = root_value->get_type();
    const_data_block_ptr ptr = root_value.get()->get_data();

    // We cannot do array processing because we do not know the size of the base
    // type of the array yet. That's why it's vague.
    assert(subparts.size() == 0);

    return ptr;
}

}
}
