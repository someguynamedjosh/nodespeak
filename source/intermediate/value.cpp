#include <waveguide/intermediate/value.hpp>

#include <waveguide/intermediate/data_type.hpp>
#include <cassert>
#include <cstring>
#include <iterator>
#include <sstream>
#include <string>

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// Com::value
////////////////////////////////////////////////////////////////////////////////

value::value(std::shared_ptr<const data_type> type)
    : type{type} {
    if (!type->is_proxy_type()) {
        data = SP<char[]>{new char[type->get_length()]};
    }
}

value::value(std::shared_ptr<const data_type> type, SP<char[]> data)
    : type{type}, data{data}, value_known{true} {
    assert(!type->is_proxy_type());
}

value::value(std::shared_ptr<const data_type> type, SP<value> target)
    : type{type}, value_known{false} {
    assert(type->is_proxy_type());
    data = std::reinterpret_pointer_cast<char[]>(target);
}

std::shared_ptr<const data_type> value::get_type() const {
    return type;
}

void value::set_type(std::shared_ptr<const data_type> new_type) {
    assert(new_type->get_length() == type->get_length());
    assert(new_type->is_proxy_type() == type->is_proxy_type());
    type = new_type;
}

bool value::is_proxy() const {
    return type->is_proxy_type();
}

value const&value::get_real_value() const {
    if (is_proxy()) {
        return std::reinterpret_pointer_cast<const value>(data)->get_real_value();
    } else {
        return *this;
    }
}

bool value::is_value_known() const {
    return is_proxy() ? get_real_value().is_value_known() : value_known;
}

void value::set_value_known(bool is_known) {
    assert(!is_proxy());
    value_known = is_known;
}

value value::create_known_copy() const {
    assert(value_known);
    value tr{type};
    auto tr_data = tr.get_data().get();
    for (int i = 0; i < type->get_length(); i++) {
        tr_data[i] = data[i];
    }
    tr.set_value_known(true);
    return tr;
}

const SP<char[]> value::get_data() const {
    assert(!is_proxy());
    return data;
}

const SP<float> value::data_as_float() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const float_data_type>(type));
    return std::reinterpret_pointer_cast<float>(data);
}

const SP<int> value::data_as_int() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const int_data_type>(type));
    return std::reinterpret_pointer_cast<int>(data);
}

const SP<bool> value::data_as_bool() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const bool_data_type>(type));
    return std::reinterpret_pointer_cast<bool>(data);
}

SP<char[]> value::get_data() {
    assert(!is_proxy());
    return data;
}

SP<float> value::data_as_float() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const float_data_type>(type));
    return std::reinterpret_pointer_cast<float>(data);
}

SP<int> value::data_as_int() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const int_data_type>(type));
    return std::reinterpret_pointer_cast<int>(data);
}

SP<bool> value::data_as_bool() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const bool_data_type>(type));
    return std::reinterpret_pointer_cast<bool>(data);
}

}
}