#include "util.hpp"

#include <waveguide/intermediate/builtins.hpp>
#include <waveguide/intermediate/scope.hpp>
#include <waveguide/intermediate/data_type.hpp>

namespace waveguide {
namespace intermediate {

SP<data_type> biggest_type(SP<data_type> a, SP<data_type> b) {
    // If A has more array dimensions, it is bigger.
    if (a->get_array_depth() > b->get_array_depth()) {
        return a;
    } else if (b->get_array_depth() > a->get_array_depth()) {
        return b;
    } 
    auto a_base = a->get_base_type(), b_base = b->get_base_type();
    // If they have the same base type, pick based on how many array
    // elements the type has in its last dimension.
    if (a_base == b_base) {
        // Check that the types actually have array depth.
        // (If B had different depth than A, earlier branch takes care of
        // it.)
        if (a->get_array_depth() > 0) {
            // If there is array depth, return whichever one has more
            // elements (in the highest-level array, not overall.)
            auto a_array = std::dynamic_pointer_cast<array_data_type>(a),
                b_array = std::dynamic_pointer_cast<array_data_type>(b);
            if (a_array->get_array_length() > b_array->get_array_length()) {
                return a;
            } else {
                return b;
            }
        // If they are not arrays, then they are just straight-up the same
        // type. Just return one of the copies.
        } else {
            return a;
        }
    } 
    int a_val = -1, b_val = -1;
    if (a_base == blt()->BOOL) a_val = 0;
    else if (a_base == blt()->INT) a_val = 1;
    else if (a_base == blt()->FLOAT) a_val = 2;
    if (b_base == blt()->BOOL) a_val = 0;
    else if (b_base == blt()->INT) a_val = 1;
    else if (b_base == blt()->FLOAT) a_val = 2;
    if (a_val > b_val) {
        return a;
    } else { // Either B is bigger, or they are the same.
        return b;
    }
}

SP<builtins> blt() {
    return builtins::get_instance();
}

}
}